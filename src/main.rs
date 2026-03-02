use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, thread, process::Command, path::Path};

fn get_bridge_ip() -> String {
    // Get the IP address of the bridge interface (Linux-specific)
    let docker_env = std::env::var("DOCKER").unwrap_or_default();
    let libvirt_env = std::env::var("LIBVIRT").unwrap_or_default();
    
    let bridges: &[&str] = if docker_env == "1" {
        &["docker0", "br0"]
    } else if libvirt_env == "1" {
        &["virbr0"]
    } else {
        &["virbr0"]
    };
    
    for bridge in bridges {
        // Check if bridge exists
        let bridge_path = format!("/sys/class/net/{}", bridge);
        if !Path::new(&bridge_path).exists() {
            continue;
        }
        
        // Use 'ip' command to get IPv4 address
        let output = match Command::new("ip")
            .args(["-4", "addr", "show", bridge])
            .output() {
            Ok(out) => out,
            Err(_) => continue,
        };
        
        if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
            // Parse output like: "inet 192.168.122.1/24 brd ..."
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("inet ") {
                    if let Some(ip_with_mask) = line.split_whitespace().nth(1) {
                        if let Some(ip) = ip_with_mask.split('/').next() {
                            return ip.to_string();
                        }
                    }
                }
            }
        }
    }
    panic!("No suitable bridge interface found");
}

fn handle_connection(mut stream: TcpStream) {
    // Stack-allocated buffer for reading request line
    let mut buffer = [0u8; 2048];
    let mut total_read = 0;
    
    // Read until we find newline
    loop {
        match stream.read(&mut buffer[total_read..]) {
            Ok(0) => return, // Connection closed
            Ok(n) => {
                total_read += n;
                // Check if we have a newline
                if let Some(pos) = buffer[..total_read].iter().position(|&b| b == b'\n') {
                    total_read = pos;
                    break;
                }
                if total_read >= buffer.len() {
                    let _ = stream.write_all(b"HTTP/1.1 414 URI Too Long\r\n\r\n");
                    return;
                }
            }
            Err(_) => {
                let _ = stream.write_all(b"HTTP/1.1 400 BAD REQUEST\r\n\r\n");
                return;
            }
        }
    }
    
    // Trim trailing \r if present
    if total_read > 0 && buffer[total_read - 1] == b'\r' {
        total_read -= 1;
    }
    
    // Convert to string slice (zero-copy)
    let request_line = match std::str::from_utf8(&buffer[..total_read]) {
        Ok(s) => s,
        Err(_) => {
            let _ = stream.write_all(b"HTTP/1.1 400 BAD REQUEST\r\n\r\n");
            return;
        }
    };
    
    // Parse URL from first request line (e.g. GET /?l=example.com HTTP/1.1)
    let url = match request_line.split_whitespace().nth(1) {
        Some(url) => url,
        None => {
            println!("Invalid HTTP request");
            let _ = stream.write_all(b"HTTP/1.1 400 BAD REQUEST \r\n\r\n");
            return;
        }
    };
    // Split once on '?' to get query, then strip "l=" prefix.
    // This preserves any '?' and '&' inside the URL value itself.
    let link = match url.split_once('?')
        .and_then(|(_, query)| query.strip_prefix("l=")) {
        Some(l) if !l.is_empty() => l,
        _ => {
            println!("No link provided");
            let _ = stream.write_all(b"HTTP/1.1 400 BAD REQUEST \r\n\r\n");
            return;
        }
    };
    let _ = stream.write_all(b"HTTP/1.1 200 OK \r\n\r\n");
    let final_link = if !link.starts_with("http://") && !link.starts_with("https://") {
        format!("https://{}", link)
    } else {
        link.to_owned()
    };
    println!("Received link: {}", final_link);
    // xdg-open the link
    match std::process::Command::new("xdg-open")
        .arg(&final_link)
        .spawn() {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to open link: {}", e);
            }
        };
    let _ = stream.flush();
    let _ = stream.shutdown(std::net::Shutdown::Both);
    
}

fn main() {
    if !cfg!(target_os = "linux") {
        panic!("Linux support only");
    }

    // Parse port without collecting all args
    let mut args = std::env::args().skip(1);
    let port = if args.next().as_deref() == Some("-p") {
        args.next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(10080)
    } else {
        10080
    };

    let bridge_ip = get_bridge_ip();
    println!("Bridge IP: {}", bridge_ip);
    println!("Server listening on {} with port {}", bridge_ip, port);
    let listener = TcpListener::bind(format!("{}:{}", bridge_ip, port)).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || handle_connection(stream));
    }
}
