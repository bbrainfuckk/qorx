use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::{json, Value};

fn spawn_mcp() -> (Child, ChildStdin, BufReader<ChildStdout>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_qorx"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("qorx mcp should start");
    let stdin = child.stdin.take().expect("mcp stdin should be piped");
    let stdout = BufReader::new(child.stdout.take().expect("mcp stdout should be piped"));
    (child, stdin, stdout)
}

fn send(stdin: &mut ChildStdin, value: Value) {
    writeln!(stdin, "{}", serde_json::to_string(&value).unwrap()).unwrap();
    stdin.flush().unwrap();
}

fn recv(stdout: &mut BufReader<ChildStdout>) -> Value {
    let mut line = String::new();
    stdout
        .read_line(&mut line)
        .expect("mcp stdout should be readable");
    assert!(!line.trim().is_empty(), "mcp server returned no response");
    serde_json::from_str(line.trim()).expect("mcp response should be JSON")
}

fn send_framed(stdin: &mut ChildStdin, value: Value) {
    let body = serde_json::to_string(&value).unwrap();
    write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
    stdin.flush().unwrap();
}

fn recv_framed(stdout: &mut BufReader<ChildStdout>) -> Value {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        stdout
            .read_line(&mut line)
            .expect("framed mcp header should be readable");
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(raw) = line.strip_prefix("Content-Length:") {
            content_length = Some(raw.trim().parse::<usize>().unwrap());
        }
    }

    let length = content_length.expect("framed MCP response should include Content-Length");
    let mut body = vec![0; length];
    stdout
        .read_exact(&mut body)
        .expect("framed MCP response body should be readable");
    serde_json::from_slice(&body).expect("framed MCP response should be JSON")
}

#[test]
fn mcp_stdio_initializes_and_lists_qorx_tools() {
    let (mut child, mut stdin, mut stdout) = spawn_mcp();

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "qorx-test", "version": "0.0.0"}
            }
        }),
    );
    let init = recv(&mut stdout);
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["serverInfo"]["name"], "qorx");
    assert_eq!(init["result"]["serverInfo"]["version"], "1.0.5");
    assert_eq!(init["result"]["capabilities"]["tools"], json!({}));
    assert!(init["result"]["instructions"]
        .as_str()
        .unwrap_or_default()
        .contains("Qorx v1.0.5"));

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    );
    let tools = recv(&mut stdout);
    let names: Vec<_> = tools["result"]["tools"]
        .as_array()
        .expect("tools/list should return tools")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(names.contains(&"qorx.health".to_string()));
    assert!(names.contains(&"qorx.session".to_string()));
    assert!(names.contains(&"qorx.squeeze".to_string()));
    assert!(names.contains(&"qorx.orcl".to_string()));

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "shutdown",
            "params": {}
        }),
    );
    let shutdown = recv(&mut stdout);
    assert_eq!(shutdown["id"], 3);

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "exit"
        }),
    );
    let status = child.wait().expect("mcp process should exit cleanly");
    assert!(status.success());
}

#[test]
fn mcp_stdio_supports_content_length_framing_for_antigravity() {
    let (mut child, mut stdin, mut stdout) = spawn_mcp();

    send_framed(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "antigravity-test", "version": "0.0.0"}
            }
        }),
    );
    let init = recv_framed(&mut stdout);
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["serverInfo"]["name"], "qorx");
    assert_eq!(init["result"]["serverInfo"]["version"], "1.0.5");
    assert!(init["result"]["instructions"]
        .as_str()
        .unwrap_or_default()
        .contains("Qorx v1.0.5"));

    send_framed(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": {}
        }),
    );
    let shutdown = recv_framed(&mut stdout);
    assert_eq!(shutdown["id"], 2);

    send_framed(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "exit"
        }),
    );
    let status = child.wait().expect("mcp process should exit cleanly");
    assert!(status.success());
}
