# Projet Serveur HTTP Localhost

## Description

Ce projet consiste à créer un serveur HTTP personnalisé en Rust. Le serveur est capable de gérer des requêtes HTTP, de servir des fichiers statiques, d'exécuter des scripts CGI, et de gérer des sessions avec des cookies.

## Fonctionnalités

- **Gestion des requêtes HTTP** : Le serveur écoute les requêtes entrantes sur un port spécifique, les analyse et renvoie des réponses appropriées.
- **Multiplexage d'E/S** : Utilisation de `mio::Poll` pour surveiller plusieurs descripteurs de fichiers pour des événements comme la lecture ou l'écriture.
- **Configuration flexible** : Possibilité de configurer plusieurs serveurs avec différents ports et noms d'hôtes.
- **Gestion des routes** : Configuration des routes avec des méthodes HTTP spécifiques autorisées.
- **Pages d'erreur personnalisées** : Configuration des pages d'erreur personnalisées.
- **Limitation de la taille du corps** : Limitation de la taille du corps des requêtes pour éviter les attaques par déni de service.
- **Sessions et cookies** : Gestion des sessions utilisateur avec des cookies.
- **Scripts CGI** : Exécution de scripts CGI pour des fonctionnalités dynamiques.
- **Listage de répertoires** : Option pour lister le contenu des répertoires.

## Configuration

La configuration du serveur se fait via le fichier `config.rs`. Voici un exemple de configuration :

```rust
ServerConfig {
    host: "127.0.0.1",
    ports: vec![8080, 8081, 8082],
    custom_error_path: Some(".assets/errors_pages"),
    body_size_limit: 1000000000024,
    routes: vec![
        Route {
            url_path: "/api/update-cookie",
            methods: vec![http::Method::POST],
            handler: Some(update_cookie),
            settings: None,
        },
        // Autres routes...
    ],
}
