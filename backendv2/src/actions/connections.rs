use futures::StreamExt;
use libp2p::{noise, tcp, yamux, StreamProtocol};
use napi_derive::napi;
// use std::{error::Error, time::Duration};
use tokio::net::TcpStream;
use tokio_util::compat::FuturesAsyncReadCompatExt;

const MC_PROTOCOL: StreamProtocol = StreamProtocol::new("/mc-p2p-tunnel/1.0.0");
const MINECRAFT_SERVER: &str = "127.0.0.1:25565";

#[napi]
// 1. Change the return type to napi::Result<()>
pub async fn p2p_host() -> napi::Result<()> {
    eprintln!("p2p working running");
    // 2. Map errors using .map_err() to convert them to napi::Error
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| napi::Error::from_reason(e.to_string()))?
        .with_behaviour(|_key| libp2p_stream::Behaviour::new())
        .map_err(|e| napi::Error::from_reason(e.to_string()))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
        .build();

    let mut stream_control = swarm.behaviour().new_control();

    let mut stream_listener = stream_control
        .accept(MC_PROTOCOL)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // Parse the address into a Multiaddr first to keep it clean
    let listen_addr: libp2p::Multiaddr = "/ip4/0.0.0.0/tcp/4001"
        .parse()
        .map_err(|e: libp2p::multiaddr::Error| napi::Error::from_reason(e.to_string()))?;

    // Now pass it to listen_on
    swarm
        .listen_on(listen_addr)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    println!(
        "Host P2P Node started. Peer ID: {:?}",
        swarm.local_peer_id()
    );
    println!("Listening for players on port 4001...");

    tokio::spawn(async move {
        loop {
            swarm.select_next_some().await;
        }
    });

    // Internal loop errors can still just be printed, as they happen in a background task
    while let Some((peer_id, p2p_stream)) = stream_listener.next().await {
        println!(
            "Received connection request from player peer: {:?}",
            peer_id
        );

        tokio::spawn(async move {
            match TcpStream::connect(MINECRAFT_SERVER).await {
                Ok(mut mc_server_stream) => {
                    println!("Successfully connected player stream to local Minecraft server.");

                    let compat_p2p_stream = p2p_stream.compat();
                    let (mut p2p_reader, mut p2p_writer) = tokio::io::split(compat_p2p_stream);
                    let (mut mc_reader, mut mc_writer) = mc_server_stream.split();

                    let client_to_server = tokio::io::copy(&mut p2p_reader, &mut mc_writer);
                    let server_to_client = tokio::io::copy(&mut mc_reader, &mut p2p_writer);

                    if let Err(e) = tokio::try_join!(client_to_server, server_to_client) {
                        eprintln!("Tunnel error or player disconnected: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to local Minecraft server: {:?}", e);
                }
            }
        });
    }

    Ok(())
}
