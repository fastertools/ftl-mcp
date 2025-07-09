use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::time::sleep;
use wasmcp::{JsonRpcRequest, JsonRpcResponse, JsonRpcId};

/// Simple test result
#[derive(Debug)]
struct TestResult {
    test_name: String,
    success: bool,
    error: Option<String>,
    duration_ms: u128,
}

/// Spin server manager
struct SpinServer {
    child: Child,
    port: u16,
}

impl SpinServer {
    async fn start(port: u16) -> Result<Self> {
        println!("{}", "Starting Spin server...".blue());
        
        let mut child = Command::new("spin")
            .arg("up")
            .arg("--listen")
            .arg(format!("127.0.0.1:{}", port))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start Spin server")?;

        // Wait for server to be ready
        let start_time = std::time::Instant::now();
        let client = reqwest::Client::new();
        
        loop {
            if start_time.elapsed().as_secs() > 30 {
                // Capture stderr output before killing
                let stderr_output = child.stderr.take();
                if let Some(mut stderr) = stderr_output {
                    let mut error_output = String::new();
                    stderr.read_to_string(&mut error_output).await.unwrap_or_default();
                    eprintln!("Spin server error output: {}", error_output);
                }
                child.kill().await?;
                anyhow::bail!("Server failed to start within 30 seconds");
            }

            // Try to connect to a known endpoint
            if let Ok(_) = client.get(format!("http://localhost:{}/", port))
                .timeout(Duration::from_secs(1))
                .send()
                .await 
            {
                println!("{}", "Server started successfully!".green());
                break;
            }

            sleep(Duration::from_millis(500)).await;
        }

        Ok(SpinServer { child, port })
    }

    async fn stop(mut self) -> Result<()> {
        println!("{}", "Stopping Spin server...".blue());
        self.child.kill().await?;
        self.child.wait().await?;
        println!("{}", "Server stopped.".green());
        Ok(())
    }
}

/// MCP Test Suite
struct McpTester {
    client: reqwest::Client,
    base_url: String,
}

impl McpTester {
    fn new(port: u16) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: format!("http://localhost:{}", port),
        }
    }
    
    fn new_with_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Send a JSON-RPC request to the MCP endpoint
    async fn send_mcp_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let url = format!("{}/mcp", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let response_text = response.text().await
            .context("Failed to get response text")?;
        
        if !status.is_success() {
            anyhow::bail!("HTTP error {}: {}", status, response_text);
        }
        
        if response_text.is_empty() {
            anyhow::bail!("Empty response body");
        }
        
        let json_response: JsonRpcResponse = serde_json::from_str(&response_text)
            .with_context(|| format!("Failed to parse JSON response: {}", response_text))?;

        Ok(json_response)
    }

    /// Send a JSON-RPC request to a specific endpoint
    async fn send_endpoint_request(&self, endpoint: &str, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self.client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let response_text = response.text().await
            .context("Failed to get response text")?;
        
        if !status.is_success() {
            anyhow::bail!("HTTP error {}: {}", status, response_text);
        }
        
        if response_text.is_empty() {
            anyhow::bail!("Empty response body");
        }
        
        let json_response: JsonRpcResponse = serde_json::from_str(&response_text)
            .with_context(|| format!("Failed to parse JSON response: {}", response_text))?;

        Ok(json_response)
    }

    /// Test ping endpoint
    async fn test_ping(&self) -> TestResult {
        let start = std::time::Instant::now();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "ping".to_string(),
            params: None,
            id: Some(JsonRpcId::Number(1)),
        };

        match self.send_mcp_request(request).await {
            Ok(response) => {
                if response.error.is_some() {
                    TestResult {
                        test_name: "ping".to_string(),
                        success: false,
                        error: response.error.map(|e| e.message),
                        duration_ms: start.elapsed().as_millis(),
                    }
                } else {
                    TestResult {
                        test_name: "ping".to_string(),
                        success: true,
                        error: None,
                        duration_ms: start.elapsed().as_millis(),
                    }
                }
            },
            Err(e) => TestResult {
                test_name: "ping".to_string(),
                success: false,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis(),
            },
        }
    }

    /// Test tools/list endpoint
    async fn test_tools_list(&self) -> TestResult {
        let start = std::time::Instant::now();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(JsonRpcId::Number(2)),
        };

        match self.send_mcp_request(request).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    TestResult {
                        test_name: "tools/list".to_string(),
                        success: false,
                        error: Some(error.message),
                        duration_ms: start.elapsed().as_millis(),
                    }
                } else if let Some(result) = response.result {
                    // Check if tools array exists and has content
                    if let Some(tools) = result.get("tools") {
                        if tools.is_array() {
                            let tools_array = tools.as_array().unwrap();
                            TestResult {
                                test_name: format!("tools/list ({} tools)", tools_array.len()),
                                success: true,
                                error: None,
                                duration_ms: start.elapsed().as_millis(),
                            }
                        } else {
                            TestResult {
                                test_name: "tools/list".to_string(),
                                success: false,
                                error: Some("Tools is not an array".to_string()),
                                duration_ms: start.elapsed().as_millis(),
                            }
                        }
                    } else {
                        TestResult {
                            test_name: "tools/list".to_string(),
                            success: false,
                            error: Some("No tools field in response".to_string()),
                            duration_ms: start.elapsed().as_millis(),
                        }
                    }
                } else {
                    TestResult {
                        test_name: "tools/list".to_string(),
                        success: false,
                        error: Some("No result in response".to_string()),
                        duration_ms: start.elapsed().as_millis(),
                    }
                }
            },
            Err(e) => TestResult {
                test_name: "tools/list".to_string(),
                success: false,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis(),
            },
        }
    }

    /// Test weather plugin directly
    async fn test_weather_direct(&self) -> TestResult {
        let start = std::time::Instant::now();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(JsonRpcId::Number(3)),
        };

        match self.send_endpoint_request("weather-new/mcp", request).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    TestResult {
                        test_name: "weather-direct".to_string(),
                        success: false,
                        error: Some(error.message),
                        duration_ms: start.elapsed().as_millis(),
                    }
                } else if let Some(result) = response.result {
                    if let Some(tools) = result.get("tools") {
                        if tools.is_array() {
                            let tools_array = tools.as_array().unwrap();
                            TestResult {
                                test_name: format!("weather-direct ({} tools)", tools_array.len()),
                                success: true,
                                error: None,
                                duration_ms: start.elapsed().as_millis(),
                            }
                        } else {
                            TestResult {
                                test_name: "weather-direct".to_string(),
                                success: false,
                                error: Some("Tools is not an array".to_string()),
                                duration_ms: start.elapsed().as_millis(),
                            }
                        }
                    } else {
                        TestResult {
                            test_name: "weather-direct".to_string(),
                            success: false,
                            error: Some("No tools field in response".to_string()),
                            duration_ms: start.elapsed().as_millis(),
                        }
                    }
                } else {
                    TestResult {
                        test_name: "weather-direct".to_string(),
                        success: false,
                        error: Some("No result in response".to_string()),
                        duration_ms: start.elapsed().as_millis(),
                    }
                }
            },
            Err(e) => TestResult {
                test_name: "weather-direct".to_string(),
                success: false,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis(),
            },
        }
    }

    /// Test weather tool call via router
    async fn test_weather_tool_call(&self) -> TestResult {
        let start = std::time::Instant::now();
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "get_weather",
                "arguments": {
                    "zipcode": "90210"
                }
            })),
            id: Some(JsonRpcId::Number(1)),
        };
        
        match self.send_mcp_request(request).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    TestResult {
                        test_name: "weather-tool-call".to_string(),
                        success: false,
                        error: Some(format!("Tool call error: {}", error.message)),
                        duration_ms: start.elapsed().as_millis(),
                    }
                } else if let Some(result) = response.result {
                    // Check if we got a proper tool result
                    if let Some(content) = result.get("content") {
                        if content.is_array() {
                            let content_array = content.as_array().unwrap();
                            if !content_array.is_empty() {
                                TestResult {
                                    test_name: "weather-tool-call".to_string(),
                                    success: true,
                                    error: None,
                                    duration_ms: start.elapsed().as_millis(),
                                }
                            } else {
                                TestResult {
                                    test_name: "weather-tool-call".to_string(),
                                    success: false,
                                    error: Some("Empty content array".to_string()),
                                    duration_ms: start.elapsed().as_millis(),
                                }
                            }
                        } else {
                            TestResult {
                                test_name: "weather-tool-call".to_string(),
                                success: false,
                                error: Some("Content is not an array".to_string()),
                                duration_ms: start.elapsed().as_millis(),
                            }
                        }
                    } else {
                        TestResult {
                            test_name: "weather-tool-call".to_string(),
                            success: false,
                            error: Some("No content field in response".to_string()),
                            duration_ms: start.elapsed().as_millis(),
                        }
                    }
                } else {
                    TestResult {
                        test_name: "weather-tool-call".to_string(),
                        success: false,
                        error: Some("No result in response".to_string()),
                        duration_ms: start.elapsed().as_millis(),
                    }
                }
            },
            Err(e) => TestResult {
                test_name: "weather-tool-call".to_string(),
                success: false,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis(),
            },
        }
    }

    /// Test activity tool call via router
    async fn test_activity_tool_call(&self) -> TestResult {
        let start = std::time::Instant::now();
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "random_activity",
                "arguments": {}
            })),
            id: Some(JsonRpcId::Number(1)),
        };
        
        match self.send_mcp_request(request).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    TestResult {
                        test_name: "activity-tool-call".to_string(),
                        success: false,
                        error: Some(format!("Tool call error: {}", error.message)),
                        duration_ms: start.elapsed().as_millis(),
                    }
                } else if let Some(result) = response.result {
                    // Check if we got a proper tool result
                    if let Some(content) = result.get("content") {
                        if content.is_array() {
                            let content_array = content.as_array().unwrap();
                            if !content_array.is_empty() {
                                TestResult {
                                    test_name: "activity-tool-call".to_string(),
                                    success: true,
                                    error: None,
                                    duration_ms: start.elapsed().as_millis(),
                                }
                            } else {
                                TestResult {
                                    test_name: "activity-tool-call".to_string(),
                                    success: false,
                                    error: Some("Empty content array".to_string()),
                                    duration_ms: start.elapsed().as_millis(),
                                }
                            }
                        } else {
                            TestResult {
                                test_name: "activity-tool-call".to_string(),
                                success: false,
                                error: Some("Content is not an array".to_string()),
                                duration_ms: start.elapsed().as_millis(),
                            }
                        }
                    } else {
                        TestResult {
                            test_name: "activity-tool-call".to_string(),
                            success: false,
                            error: Some("No content field in response".to_string()),
                            duration_ms: start.elapsed().as_millis(),
                        }
                    }
                } else {
                    TestResult {
                        test_name: "activity-tool-call".to_string(),
                        success: false,
                        error: Some("No result in response".to_string()),
                        duration_ms: start.elapsed().as_millis(),
                    }
                }
            },
            Err(e) => TestResult {
                test_name: "activity-tool-call".to_string(),
                success: false,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis(),
            },
        }
    }

    /// Run all tests
    async fn run_tests(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        // Test 1: Ping
        println!("Running test: {}", "ping".yellow());
        results.push(self.test_ping().await);
        
        // Test 2: Weather plugin direct access
        println!("Running test: {}", "weather-direct".yellow());
        results.push(self.test_weather_direct().await);
        
        // Test 3: Router tools/list
        println!("Running test: {}", "tools/list via router".yellow());
        results.push(self.test_tools_list().await);
        
        // Test 4: Weather tool call via router
        println!("Running test: {}", "weather tool call via router".yellow());
        results.push(self.test_weather_tool_call().await);
        
        // Test 5: Activity tool call via router
        println!("Running test: {}", "activity tool call via router".yellow());
        results.push(self.test_activity_tool_call().await);
        
        results
    }
}

/// Print test results
fn print_results(results: &[TestResult]) {
    println!("\n{}", "Test Results:".blue().bold());
    println!("{}", "=============".blue());
    
    let mut passed = 0;
    let mut failed = 0;
    
    for result in results {
        if result.success {
            println!("{} {} ({}ms)", 
                "PASS".green().bold(), 
                result.test_name, 
                result.duration_ms
            );
            passed += 1;
        } else {
            println!("{} {} ({}ms)", 
                "FAIL".red().bold(), 
                result.test_name, 
                result.duration_ms
            );
            if let Some(ref error) = result.error {
                println!("  └─ Error: {}", error.red());
            }
            failed += 1;
        }
    }
    
    println!("\n{}: {}/{} tests passed", 
        "Summary".blue().bold(), 
        passed, 
        passed + failed
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "MCP Test Runner - Simple Incremental Testing".blue().bold());
    println!();
    
    let args: Vec<String> = env::args().collect();
    
    // Check for URL argument
    let external_url = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };
    
    match external_url {
        Some(url) => {
            // Test external URL (no local server)
            println!("Testing external URL: {}", url.cyan());
            let tester = McpTester::new_with_url(url);
            let results = tester.run_tests().await;
            print_results(&results);
            
            // Exit with error code if any tests failed
            let failed_count = results.iter().filter(|r| !r.success).count();
            if failed_count > 0 {
                std::process::exit(1);
            }
        }
        None => {
            // Test locally (start server)
            println!("Testing locally on port 3000");
            let port = 3000;
            
            // Start Spin server
            let server = SpinServer::start(port).await?;
            
            // Run tests (server will be cleaned up in any case)
            let result = async {
                let tester = McpTester::new(port);
                let results = tester.run_tests().await;
                print_results(&results);
                results
            }.await;
            
            // Always stop server
            server.stop().await?;
            
            // Exit with error code if any tests failed
            let failed_count = result.iter().filter(|r| !r.success).count();
            if failed_count > 0 {
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}