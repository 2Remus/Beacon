use futures::StreamExt;
use libp2p::{Multiaddr, PeerId, StreamProtocol, noise, tcp, yamux};
use std::error::Error;
use tokio::net::TcpListener;
use tokio_util::compat::FuturesAsyncReadCompatExt;

const MC_PROTOCOL: StreamProtocol = StreamProtocol::new("/mc-p2p-tunnel/1.0.0");
// The local port your Minecraft game will connect to
const LOCAL_MINECRAFT_PORT: &str = "127.0.0.1:25566";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Get Host details from arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin p2p_test_client <HOST_MULTIADDR>");
        eprintln!(
            "Example: cargo run --bin p2p_test_client /ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."
        );
        std::process::exit(1);
    }

    let host_target: Multiaddr = args[1].parse()?;

    // 2. Setup the Libp2p Client Swarm
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_key| libp2p_stream::Behaviour::new())?
        .build();

    // 3. Get the stream control handle before moving the swarm
    let mut stream_control = swarm.behaviour().new_control();

    // 4. Dial the Host Node
    println!("Dialing P2P Host at: {}...", host_target);
    swarm.dial(host_target.clone())?;

    // Extract the PeerId from the multiaddr to open streams later
    let host_peer_id = host_target
        .iter()
        .find_map(|p| match p {
            libp2p::multiaddr::Protocol::P2p(peer_id) => Some(peer_id),
            _ => None,
        })
        .ok_or("Multiaddr must include the host's /p2p/<PeerId> at the end!")?;

    // 5. Drive the swarm in a background task
    tokio::spawn(async move {
        loop {
            swarm.select_next_some().await;
        }
    });

    // 6. Bind a local TCP port for your Minecraft game client to join
    let listener = TcpListener::bind(LOCAL_MINECRAFT_PORT).await?;
    println!("\n🚀 P2P Test Client Ready!");
    println!("👉 Open Minecraft and connect to: {}", LOCAL_MINECRAFT_PORT);

    // 7. Listen for local Minecraft game connections and tunnel them over P2P
    loop {
        let (mut game_stream, _) = listener.accept().await?;
        println!("Game client connected! Opening P2P stream to host...");

        let mut control = stream_control.clone();

        tokio::spawn(async move {
            // Open a new outbound stream using our custom protocol over P2P
            match control.open_stream(host_peer_id, MC_PROTOCOL).await {
                Ok(p2p_stream) => {
                    println!("P2P stream successfully established to host.");

                    let compat_p2p_stream = p2p_stream.compat();
                    let (mut p2p_reader, mut p2p_writer) = tokio::io::split(compat_p2p_stream);
                    let (mut game_reader, mut game_writer) = game_stream.split();

                    // Conduit between Minecraft Game -> Client P2P -> Host P2P -> Server
                    let game_to_p2p = tokio::io::copy(&mut game_reader, &mut p2p_writer);
                    let p2p_to_game = tokio::io::copy(&mut p2p_reader, &mut game_writer);

                    if let Err(e) = tokio::try_join!(game_to_p2p, p2p_to_game) {
                        eprintln!("Tunnel disconnected: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open P2P stream to host: {:?}", e);
                }
            }
        });
    }
}
