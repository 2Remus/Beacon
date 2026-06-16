use futures::StreamExt;
use libp2p::{noise, tcp, yamux, StreamProtocol};
use std::error::Error;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_util::compat::FuturesAsyncReadCompatExt;

// Define a unique protocol name for your Minecraft P2P traffic
const MC_PROTOCOL: StreamProtocol = StreamProtocol::new("/mc-p2p-tunnel/1.0.0");
const MINECRAFT_SERVER: &str = "127.0.0.1:25565";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup the Libp2p Swarm
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        // Fix: Use the official stream behaviour instead of dummy
        .with_behaviour(|_key| libp2p_stream::Behaviour::new())?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
        .build();

    // 2. Register our custom Minecraft protocol listener
    // Fix: Use the new control stream handle to accept the protocol
    let mut stream_control = swarm.behaviour().new_control();
    let mut stream_listener = stream_control.accept(MC_PROTOCOL)?;

    // 3. Listen on a public interface for incoming player connections
    swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?)?;
    println!(
        "Host P2P Node started. Peer ID: {:?}",
        swarm.local_peer_id()
    );
    println!("Listening for players on port 4001...");

    // 4. Drive the swarm in a background task
    tokio::spawn(async move {
        loop {
            // Fix: Modern libp2p drives itself automatically when you await the stream next
            swarm.select_next_some().await;
        }
    });

    // 5. Main loop: Handle incoming P2P streams from players
    while let Some((peer_id, p2p_stream)) = stream_listener.next().await {
        println!(
            "Received connection request from player peer: {:?}",
            peer_id
        );

        // Spawn a task to bridge this specific player to the Minecraft server
        tokio::spawn(async move {
            // Connect to the actual local Minecraft server
            match TcpStream::connect(MINECRAFT_SERVER).await {
                Ok(mut mc_server_stream) => {
                    println!("Successfully connected player stream to local Minecraft server.");

                    let compat_p2p_stream = p2p_stream.compat();

                    // Split both streams into read/write halves
                    let (mut p2p_reader, mut p2p_writer) = tokio::io::split(compat_p2p_stream);
                    let (mut mc_reader, mut mc_writer) = mc_server_stream.split();

                    // Concurrently copy bytes back and forth (The Tunnel)
                    let client_to_server = tokio::io::copy(&mut p2p_reader, &mut mc_writer);
                    let server_to_client = tokio::io::copy(&mut mc_reader, &mut p2p_writer);

                    if let Err(e) = tokio::try_join!(client_to_server, server_to_client) {
                        eprintln!("Tunnel error or player disconnected: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to connect to local Minecraft server (is it running?): {:?}",
                        e
                    );
                }
            }
        });
    }

    Ok(())
}
