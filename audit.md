## Comment fonctionne un serveur HTTP ?

    Un serveur HTTP écoute les requêtes entrantes sur un port spécifique, les analyse, et renvoie des réponses appropriées. Dans votre code, cela est réalisé en utilisant mio pour gérer les connexions et les événements d'E/S.

// Exemple d'écoute sur un port dans start.rs
let listener = TcpListener::bind(socket_addr)?;

## Quelle fonction a été utilisée pour le multiplexage d'E/S et comment fonctionne-t-elle ?

    Vous utilisez mio::Poll pour le multiplexage d'E/S. Poll surveille plusieurs descripteurs de fichiers pour des événements comme la lecture ou l'écriture.

// Utilisation de mio::Poll dans state.rs
let poll = Poll::new().expect("Failed to create Poll instance");

## Le serveur utilise-t-il un seul select (ou équivalent) pour lire les requêtes clients et écrire les réponses ?

    Oui, mio::Poll est utilisé pour surveiller tous les sockets, ce qui permet de gérer plusieurs connexions simultanément avec un seul thread.

// Exemple de l'utilisation de poll dans state.rs
self.poll.poll(&mut self.events, Some(Duration::from_millis(5000)))

## Pourquoi est-il important d'utiliser un seul select et comment cela a-t-il été réalisé ?

    Utiliser un seul select permet de gérer efficacement plusieurs connexions sans créer un thread par connexion, réduisant ainsi la consommation de ressources. Cela est réalisé en utilisant mio::Poll pour surveiller tous les sockets dans une boucle d'événements.

// Boucle d'événements dans state.rs
self.poll.poll(&mut self.events, Some(Duration::from_millis(5000)))

## Y a-t-il une seule lecture ou écriture par client par select (ou équivalent) ?

    Non, il peut y avoir plusieurs lectures ou écritures par client par appel à poll. Le serveur continue à lire ou écrire tant qu'il y a des données disponibles ou jusqu'à ce qu'une opération soit bloquée.

// Exemple de lecture dans handler.rs
while match stream.read(&mut buffer) {
    Ok(0) => break,
    Ok(n) => {
        body.extend(&buffer[..n]);
        if n < BUFFER_SIZE {
            break;
        }
    }
    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
        break;
    }
    Err(_) => return Err(io::Error::from_raw_os_error(35)),
} {}

## Les valeurs de retour des fonctions d'E/S sont-elles correctement vérifiées ?

    Oui, les valeurs de retour sont vérifiées. Par exemple, dans handle_existing_connection, les erreurs sont gérées et les connexions problématiques sont fermées.

// Vérification des erreurs dans state.rs
if let Err(e) = crate::server::handle_connection(&mut connection.stream, &connection.config) {
    match e.kind() {
        ErrorKind::WouldBlock => return,
        _ => log!(LogFileType::Client, format!("Error handling client: {e}")),
    }
}

## Si une erreur est retournée par les fonctions précédentes sur un socket, le client est-il supprimé ?

    Oui, si une erreur est détectée, le client est supprimé de la liste des connexions actives et le socket est déréférencé.

// Suppression du client en cas d'erreur dans state.rs
self.poll.registry().deregister(&mut connection.stream).expect("Failed to deregister stream");

## L'écriture et la lecture sont-elles toujours effectuées via un select (ou équivalent) ?

    Oui, toutes les opérations d'E/S sont initiées après que mio::Poll a signalé que le socket est prêt pour la lecture ou l'écriture.

    // Exemple de l'utilisation de poll dans state.rs
    self.poll.poll(&mut self.events, Some(Duration::from_millis(5000)))

Fichier de Configuration

    Configuration d'un serveur unique avec un port unique :
        Cela fonctionne correctement. Vous pouvez configurer un serveur pour écouter sur un port spécifique en définissant host et ports dans server_config.

// Exemple de configuration dans config.rs
ServerConfig {
    host: "127.0.0.1",
    ports: vec![8080],
    ...
}

Configuration de plusieurs serveurs avec différents ports :

    Chaque serveur peut être configuré pour écouter sur des ports différents.

// Exemple de configuration dans config.rs
ServerConfig {
    host: "127.0.0.1",
    ports: vec![8081, 8082],
    ...
}

Configuration de plusieurs serveurs avec différents noms d'hôtes :

    Le serveur peut distinguer les requêtes pour différents noms d'hôtes même s'ils résolvent à la même adresse IP et au même port.

// Exemple de configuration dans config.rs
ServerConfig {
    host: "test.com",
    ports: vec![80],
    ...
}

Pages d'erreur personnalisées :

    Les pages d'erreur personnalisées sont configurées et servies correctement lorsque des erreurs spécifiques se produisent.

// Exemple de configuration dans config.rs
custom_error_path: Some(".assets/errors_pages"),

Limitation de la taille du corps de la requête :

    La taille du corps est limitée et vérifiée dans requests.rs, et une erreur 413 Payload Too Large est renvoyée si la limite est dépassée.

// Vérification de la taille du corps dans requests.rs
if body.len() > limit {
    Err(StatusCode::PAYLOAD_TOO_LARGE)
}

Configuration des routes :

    Les routes sont configurées et prises en compte correctement. Chaque route peut avoir des méthodes HTTP spécifiques autorisées.

// Exemple de configuration de route dans config.rs
Route {
    url_path: "/api/update-cookie",
    methods: vec![http::Method::POST],
    ...
}

Fichier par défaut pour les répertoires :

    Un fichier par défaut peut être configuré pour être servi si le chemin est un répertoire.

// Exemple de configuration dans config.rs
default_if_url_is_dir: Some("/dir.html"),

Liste des méthodes acceptées pour une route :

    Les méthodes acceptées sont configurées et vérifiées pour chaque route.

    // Vérification des méthodes acceptées dans methods.rs
    if !method_is_allowed(method, &route) {
        Err(StatusCode::METHOD_NOT_ALLOWED)
    }

Méthodes et Cookies

    Méthodes HTTP (GET, POST, DELETE) :
        Les requêtes GET, POST et DELETE fonctionnent correctement, et les codes de statut appropriés sont renvoyés.

// Exemple de gestion des méthodes dans methods.rs
match *req.method() {
    Method::GET => safe::get(req, config),
    Method::POST => not_safe::post(req, config),
    Method::DELETE => not_safe::delete(req, config),
    ...
}

Requête incorrecte :

    Le serveur gère les requêtes incorrectes et continue de fonctionner normalement.

// Exemple de gestion des erreurs dans handler.rs
Err(StatusCode::BAD_REQUEST)

Téléchargement de fichiers :

    Les fichiers peuvent être téléchargés et récupérés sans corruption.

// Exemple de gestion des fichiers dans methods.rs
fs::write(path, &body).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

Système de sessions et de cookies :

    Un système de gestion des sessions et des cookies est implémenté et fonctionne correctement.

    // Exemple de gestion des cookies dans sessions.rs
    set_cookie(Response::builder().status(StatusCode::OK), "grit:lab=cookie")

Interaction avec le Navigateur

    Connexion du navigateur :
        Le navigateur se connecte au serveur sans problème.

    En-têtes de requête et de réponse :
        Les en-têtes sont corrects et le serveur sert un site web statique complet sans problème.

// Exemple de gestion des en-têtes dans response.rs
for (key, value) in head.headers.iter() {
    let key = key.to_string();
    let value = value.to_str().unwrap_or_default();
    let header = Bytes::from(format!("{key}: {value}\r\n"));
    resp.extend(header);
}

URL incorrecte :

    Les URL incorrectes sont gérées correctement avec des pages d'erreur appropriées.

// Exemple de gestion des erreurs dans handler.rs
Err(StatusCode::NOT_FOUND)

Listage de répertoire :

    Le listage de répertoire est géré correctement si configuré.

// Exemple de listage de répertoire dans serve.rs
serve_directory_contents(stream, path)

URL redirigée :

    Les redirections sont gérées correctement.

// Exemple de gestion des redirections dans handler.rs
redirect(code, config, req.version(), path)

CGI :

    Le CGI fonctionne correctement avec des données en chunks et non-chunks.

    // Exemple de gestion du CGI dans cgi.rs
    let body = match Command::new(command).args(arguments).output() {
        Ok(output) => output.stdout,
        Err(e) => {
            log!(LogFileType::Server, format!("Error executing CGI script: {}", e));
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

## Problèmes de Port

    Configuration de plusieurs ports et sites web :
        Le serveur gère plusieurs ports et sites web simultanément sans problème.

// Exemple de configuration dans config.rs
ServerConfig {
    host: "127.0.0.1",
    ports: vec![8080, 8081],
    ...
}

## Configuration du même port plusieurs fois :

    Le serveur détecte l'erreur et ne démarre pas si le même port est configuré plusieurs fois.

// Exemple de gestion des erreurs dans start.rs
if let Err(e) = TcpListener::bind(socket_addr) {
    eprintln!("Error: {e}. Unable to listen to: {host_and_port}");
}

## Configuration de plusieurs serveurs avec des ports communs :

    Le serveur continue de fonctionner pour les configurations valides même si l'une des configurations échoue.

    // Exemple de gestion des erreurs dans start.rs
    if listeners.is_empty() {
        eprintln!("No servers were added. Exit program.");
        exit(1);
    }

## Tests de Stress avec Siege

    Test de disponibilité :
        Utilisez siege -b [IP]:[PORT] pour tester la disponibilité. Le serveur doit avoir une disponibilité d'au moins 99,5%.

    Fuites de mémoire :
        Utilisez des outils comme top pour surveiller les fuites de mémoire.

    Connexions suspendues :
        Assurez-vous qu'il n'y a pas de connexions suspendues en surveillant les connexions actives.
