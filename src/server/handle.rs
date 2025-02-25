use crate::log;
use crate::log::*;
use crate::server::errors::error;
use crate::server::handle_method;
use crate::server::path::add_root_to_path;
use crate::server::redirections::redirect;
use crate::server::safe::get;
use crate::server::*;
use serve::*;
use std::path::Path;

const KB: usize = 1024;
pub const BUFFER_SIZE: usize = KB;

// Fonction principale pour gérer une connexion client
pub fn handle_connection(stream: &mut TcpStream, config: &ServerConfig) -> io::Result<()> {
    // Analyser la requête HTTP
    let request_parts = unsafe { parse_http_request(stream) }.map_err(|_| io::Error::from_raw_os_error(35))?;
    let request = get_request(config, request_parts.clone())
        .map_err(|e| serve_response(stream, error(e, config)))
        .unwrap_or_else(|_| Default::default());

    // Obtenir la route correspondant à la requête
    let route = match get_route(&request, config) {
        Ok(route) => route,

        // Gérer les redirections
        Err((code, path)) if code.is_redirection() => {
            return serve_response(stream, redirect(code, config, request.version(), path));
        }

        // Gérer les erreurs
        Err((code, _)) => {
            log!(LogFileType::Server, format!("Error: {}", &code));
            return serve_response(stream, error(code, config));
        }
    };

    // Utiliser le gestionnaire associé à la route
    if let Some(handler) = route.handler {
        return match handler(&request, config) {
            Ok(response) => serve_response(stream, response),
            Err(code) => {
                log!(LogFileType::Server, format!("Error: {}", &code));
                serve_response(stream, error(code, config))
            }
        };
    }

    let path = &add_root_to_path(&route, request.uri().path());

    // Vérifier si le chemin est un répertoire et si un fichier par défaut est spécifié
    if Path::new(&path).is_dir() && route.settings.is_some() {
        let settings = route.settings.as_ref().unwrap();

        // Servir le fichier par défaut si activé dans la configuration
        if let Some(default_file) = settings.default_if_url_is_dir {
            let default_path = &add_root_to_path(&route, default_file);
            let new_head = replace_path_in_request(request_parts.0, request.uri().path(), default_path);
            let request_parts = (new_head, request_parts.1);
            let request = match get_request(config, request_parts) {
                Ok(r) => r,
                Err(code) => {
                    log!(LogFileType::Server, code.to_string());
                    return serve_response(stream, error(code, config));
                }
            };

            return match get(&request, config) {
                Ok(resp) => serve_response(stream, resp),
                Err(e) => serve_response(stream, error(e, config)),
            };
        }

        // Lister le contenu du répertoire si activé
        return if settings.list_directory {
            serve_directory_contents(stream, path)
        } else {
            serve_response(stream, error(StatusCode::NOT_FOUND, config))
        };
    }

    // Vérifier si la requête est destinée à un script CGI
    if is_cgi_request(path) {
        return match execute_cgi_script(&request, config) {
            Ok(resp) => serve_response(stream, resp),
            Err(code) => {
                log!(LogFileType::Server, format!("Error: {}", &code));
                serve_response(stream, error(code, config))
            }
        };
    }

    // Gérer la méthode HTTP
    match handle_method(&route, &request, config) {
        Ok(response) => serve_response(stream, response),
        Err(code) => {
            log!(LogFileType::Server, format!("Error: {}", &code));
            serve_response(stream, error(code, config))
        }
    }
}

// Fonction pour analyser une requête HTTP
unsafe fn parse_http_request(stream: &mut TcpStream) -> Result<(String, Vec<u8>), u32> {
    let mut buffer = [0; BUFFER_SIZE];
    let mut head = String::new();
    let mut body = Vec::new();

    // Lire l'en-tête et les premiers octets du corps
    loop {
        let bytes_read = stream.read(&mut buffer).map_err(|_| line!())?;

        if bytes_read == 0 {
            return Ok((head, body));
        }

        match String::from_utf8(buffer[..bytes_read].to_vec()) {
            Ok(chunk) => {
                if let Some(index) = chunk.find("\r\n\r\n") {
                    // Séparer l'en-tête et le corps lorsque le double CRLF est trouvé
                    head.push_str(&chunk[..index]);
                    body.extend(&buffer[index + 4..bytes_read]);
                    break;
                } else {
                    // Si aucun double CRLF trouvé, ajouter le morceau entier à l'en-tête
                    head.push_str(&chunk);
                }
            }
            Err(_) => {
                let rest;
                unsafe {
                    rest = String::from_utf8_unchecked(buffer.to_vec());
                }
                let index = rest.find("\r\n\r\n").unwrap_or(0);
                head.push_str(rest.split_at(index).0);
                if index == 0 {
                    body.extend(&buffer[index..bytes_read]);
                } else {
                    body.extend(&buffer[index + 4..bytes_read]);
                }
                break;
            }
        }
        // Vider le buffer
    }

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(b) => b,
            Err(_) => return Ok((head, body)),
        };
        body.extend(buffer);
        if bytes_read < BUFFER_SIZE {
            break;
        }
    }

    Ok((head, body))
}

// Fonction pour remplacer le chemin dans une requête
fn replace_path_in_request(head: String, path: &str, default_path: &str) -> String {
    return if let Some(stripped_path) = path.strip_prefix('.') {
        head.replacen(stripped_path, &default_path[1..], 1)
    } else {
        head.replacen(path, &default_path[1..], 1)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_path_with_stripped_prefix() {
        let head = "GET /old_path HTTP/1.1\r\n".to_string();
        let path = "./old_path";
        let default_path = "/new_path";

        let result = replace_path_in_request(head, path, default_path);
        assert_eq!(result, "GET new_path HTTP/1.1\r\n");
    }

    #[test]
    fn replace_path_without_stripped_prefix() {
        let head = "POST /old_path HTTP/1.1\r\n".to_string();
        let path = "/old_path";
        let default_path = "/new_path";

        let result = replace_path_in_request(head, path, default_path);
        assert_eq!(result, "POST new_path HTTP/1.1\r\n");
    }

    #[test]
    fn replace_path_not_found() {
        let head = "PUT /another_path HTTP/1.1\r\n".to_string();
        let path = "/old_path";
        let default_path = "/new_path";

        let result = replace_path_in_request(head, path, default_path);
        assert_eq!(result, "PUT /another_path HTTP/1.1\r\n");
    }
}

mod serve {
    use crate::server::format_response;
    use crate::type_aliases::Bytes;
    use http::header::CONTENT_TYPE;
    use http::{Response, StatusCode};
    use mio::net::TcpStream;
    use std::io::Write;
    use std::path::Path;
    use std::{fs, io};

    // Fonction pour envoyer une réponse au client
    pub fn serve_response(stream: &mut TcpStream, response: Response<Bytes>) -> io::Result<()> {
        let formatted_response = format_response(response.clone());
        let total_size = formatted_response.len();
        let mut written_size = 0;

        while written_size < total_size {
            match stream.write(&formatted_response[written_size..]) {
                Ok(0) => {
                    break; // Plus de données à écrire
                }
                Ok(n) => {
                    written_size += n;
                }
                Err(e) if e.kind() != io::ErrorKind::WouldBlock => {
                    return Err(e); // Erreur n'est pas WouldBlock, retourner l'erreur.
                }
                _ => {} // Événement est maintenant bloquant, réessayer plus tard.
            }
        }

        stream.flush()
    }

    // Fonction pour servir le contenu d'un répertoire
    pub fn serve_directory_contents(stream: &mut TcpStream, path: &str) -> io::Result<()> {
        // S'assurer que le chemin ne se termine pas par un slash
        let trimmed_path = path.trim_end_matches('/');

        let base_path = Path::new(trimmed_path);
        let entries = fs::read_dir(base_path)
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Directory not found"))?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        // Réunir toutes les entrées dans une liste non ordonnée
        let body = format!(
            "<html><body><ul>{}</ul></body></html>",
            entries.into_iter().fold(String::new(), |acc, entry_path| {
                // Construire le chemin relatif à partir du chemin de base
                let relative_path = entry_path
                    .strip_prefix(base_path)
                    .unwrap_or(&entry_path)
                    .display()
                    .to_string();

                let entry_name = entry_path.file_name().unwrap_or_default().to_string_lossy();

                acc + &format!(
                    "<li><a href=\"/{}/{}\">{}</a></li>",
                    trimmed_path, relative_path, entry_name
                )
            })
        );

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/html")
            .body(Bytes::from(body))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Could not build response"))?;

        serve_response(stream, response)
    }
}
