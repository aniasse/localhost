mod server;
mod config;
mod cgi;
mod utils;

use server::Server;
use config::Config;

fn main() {
    // Charger la configuration
    let config = Config::load("config.yaml").expect("Erreur lors du chargement de la configuration");

    // Démarrer le serveur
    let mut server = Server::new(config);
    server.run().expect("Erreur lors de l'exécution du serveur");
}
