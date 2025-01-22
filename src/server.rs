use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use libc::{epoll_create1, epoll_ctl, epoll_wait, epoll_event, EPOLLIN, EPOLL_CTL_ADD};
use std::os::unix::io::{AsRawFd, RawFd};
use crate::config::Config;
use crate::cgi::execute_cgi;
use crate::utils::{parse_request, create_response};

pub struct Server {
    config: Config,
    clients: Arc<Mutex<HashMap<RawFd, TcpStream>>>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Server {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        let listener = TcpListener::bind(format!("{}:{}", self.config.host, self.config.ports[0]))
            .map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;

        let epoll_fd = unsafe { epoll_create1(0) };
        if epoll_fd == -1 {
            return Err("Erreur lors de la création de epoll".to_string());
        }

        let mut event = epoll_event {
            events: EPOLLIN as u32,
            u64: listener.as_raw_fd() as u64,
        };

        unsafe {
            epoll_ctl(epoll_fd, EPOLL_CTL_ADD, listener.as_raw_fd(), &mut event);
        }

        let mut events = vec![epoll_event { events: 0, u64: 0 }; 1024];

        loop {
            let nfds = unsafe { epoll_wait(epoll_fd, events.as_mut_ptr(), 1024, -1) };
            if nfds == -1 {
                return Err("Erreur lors de l'attente de epoll".to_string());
            }

            for i in 0..nfds {
                let fd = events[i as usize].u64 as RawFd;
                if fd == listener.as_raw_fd() {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            stream.set_nonblocking(true).unwrap();
                            let mut event = epoll_event {
                                events: EPOLLIN as u32,
                                u64: stream.as_raw_fd() as u64,
                            };
                            unsafe {
                                epoll_ctl(epoll_fd, EPOLL_CTL_ADD, stream.as_raw_fd(), &mut event);
                            }
                            self.clients.lock().unwrap().insert(stream.as_raw_fd(), stream);
                        }
                        Err(e) => eprintln!("Erreur lors de l'acceptation de la connexion : {}", e),
                    }
                } else {
                    let mut clients = self.clients.lock().unwrap();
                    if let Some(stream) = clients.get_mut(&fd) {
                        let mut buffer = [0; 1024];
                        match stream.read(&mut buffer) {
                            Ok(0) => {
                                // Connexion fermée
                                clients.remove(&fd);
                            }
                            Ok(n) => {
                                let request = String::from_utf8_lossy(&buffer[..n]);
                                let (method, path, headers, body) = parse_request(&request);
                                let response = self.handle_request(method, path, headers, body);
                                stream.write(response.as_bytes()).unwrap();
                                stream.flush().unwrap();
                            }
                            Err(e) => eprintln!("Erreur lors de la lecture : {}", e),
                        }
                    }
                }
            }
        }
    }

    fn handle_request(&self, _method: String, path: String, _headers: HashMap<String, String>, body: String) -> String {
        if path.ends_with(".php") {
            // Exécuter un script CGI
            let output = execute_cgi(&path, &body);
            create_response(200, "OK", output)
        } else {
            // Servir un fichier statique
            let file_path = format!("./static{}", path);
            match std::fs::read_to_string(&file_path) {
                Ok(content) => create_response(200, "OK", content),
                Err(_) => {
                    let error_page = std::fs::read_to_string("./static/errors/404.html").unwrap();
                    create_response(404, "Not Found", error_page)
                }
            }
        }
    }
}
