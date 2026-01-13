use crate::error::Result;
use crate::mcp::server::{McpServer, ServerConfig};
use crate::mcp::stdio_server::StdioServer;

pub async fn start(config: Option<String>, daemon: bool, stdio: bool) -> Result<()> {
    // Only enable tracing if not in STDIO mode
    if !stdio {
        tracing::info!("Starting MCP server");
    }

    let server_config = ServerConfig {
        server_name: "ktme-mcp-server".to_string(),
        transport: if stdio {
            "stdio".to_string()
        } else if daemon {
            "sse".to_string()
        } else {
            "stdio".to_string()
        },
        port: if daemon || !stdio { Some(3000) } else { None },
    };

    let server = McpServer::new(server_config)?;

    if let Some(cfg) = config {
        if !stdio {
            tracing::info!("Using config: {}", cfg);
        }
    }

    if daemon {
        tracing::info!("Running in daemon mode on SSE port 3000");
        // Only print to stdout if not in stdio mode
        if !stdio {
            println!("🚀 ktme MCP server started in daemon mode on http://localhost:3000");
            println!("💡 Configure your AI assistant to connect to: http://localhost:3000");
        }
        server.start().await
    } else if stdio {
        // Use clean STDIO server with no logging or output
        let stdio_server = StdioServer::new();
        stdio_server.run().await
    } else {
        tracing::info!("Running in STDIO mode (default)");
        println!("🚀 ktme MCP server started in STDIO mode");
        println!("💡 Ready for Claude Code integration");
        println!("💡 Use --stdio flag for clean MCP protocol output");
        server.start().await
    }
}

pub async fn status() -> Result<()> {
    tracing::info!("Checking MCP server status");

    // Try to connect to running server
    let client = reqwest::Client::new();
    let response = client.get("http://localhost:3000/status").send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))?;

            println!("✅ MCP server is running");
            println!(
                "   Status: {}",
                body.get("status")
                    .unwrap_or(&serde_json::Value::String("unknown".to_string()))
            );
            if let Some(version) = body.get("version") {
                println!("   Version: {}", version);
            }
            if let Some(tools) = body.get("tools_count") {
                println!("   Available tools: {}", tools);
            }
        }
        Ok(_) => {
            println!("❌ MCP server is not running");
            println!("   Start it with: ktme mcp start --daemon");
        }
        Err(e) => {
            println!("❌ Error connecting to MCP server: {}", e);
            println!("   Start it with: ktme mcp start --daemon");
        }
    }

    Ok(())
}

pub async fn stop() -> Result<()> {
    tracing::info!("Stopping MCP server");

    // Try to stop running server
    let client = reqwest::Client::new();
    let response = client.post("http://localhost:3000/shutdown").send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ MCP server stopped successfully");
        }
        Ok(_) => {
            println!("⚠️  MCP server may not be running or already stopped");
        }
        Err(e) => {
            println!("❌ Error stopping MCP server: {}", e);
        }
    }

    Ok(())
}
