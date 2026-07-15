# Izyploy — Spécification produit

## 1. Vision

Izyploy est une petite plateforme de déploiement qui transforme un dépôt Git public contenant un `Dockerfile` en application web accessible.

La promesse utilisateur est volontairement simple :

```text
URL du dépôt Git
        ↓
Clone et construction de l'image
        ↓
Démarrage du conteneur
        ↓
URL de l'application
```

Le projet est d'abord un support d'apprentissage du Platform Engineering. Il ne cherche pas à concurrencer Coolify, Dokploy, Railway ou Render.

## 2. Objectifs d'apprentissage

Le projet doit permettre d'apprendre progressivement :

- Rust côté backend et API HTTP ;
- le cycle de vie d'une image et d'un conteneur Docker ;
- l'exécution asynchrone de travaux longs ;
- la persistance d'état ;
- les ports, réseaux, reverse proxies, DNS et certificats TLS ;
- les logs, métriques et vérifications de santé ;
- la gestion des erreurs, le nettoyage et les redéploiements ;
- les limites de sécurité liées à l'exécution de code non fiable ;
- puis Kubernetes, un registre d'images et l'autoscaling.

## 3. Parcours utilisateur cible

1. L'utilisateur saisit le nom de l'application, l'URL d'un dépôt Git public, une branche et le port HTTP interne.
2. Izyploy crée une demande de déploiement.
3. La plateforme clone le dépôt dans un espace de travail temporaire.
4. Elle vérifie la présence d'un `Dockerfile` à la racine.
5. Elle construit une image Docker propre à ce déploiement.
6. Elle démarre un conteneur avec des limites de ressources.
7. Elle affiche l'état et les logs du déploiement.
8. Elle fournit une URL permettant d'ouvrir l'application.
9. L'utilisateur peut supprimer l'application et ses ressources.

## 4. Périmètre du premier MVP

Le premier MVP comprend :

- les dépôts GitHub publics uniquement ;
- une branche configurable, `main` par défaut ;
- un `Dockerfile` obligatoire à la racine du dépôt ;
- une seule machine équipée de Docker ;
- une API écrite en Rust avec Axum ;
- un déploiement à la fois au début ;
- des états de déploiement explicites ;
- les logs du clone, du build et du démarrage ;
- un port hôte attribué à l'application ;
- la consultation et la suppression d'une application ;
- une interface web minimale uniquement après validation de l'API.

Le premier parcours démontrable est :

```text
POST /applications
        ↓
queued → cloning → building → starting → running
        ↓
GET /applications/{id}
        ↓
http://localhost:<port>
```

## 5. Hors périmètre du premier MVP

Les éléments suivants sont volontairement reportés :

- dépôts privés et authentification GitHub ;
- détection automatique du langage ;
- génération automatique d'un `Dockerfile` ;
- multi-utilisateurs et authentification ;
- variables secrètes fournies par les utilisateurs ;
- plusieurs serveurs ou plusieurs workers ;
- haute disponibilité et autoscaling ;
- volumes persistants pour les applications déployées ;
- déploiement sans interruption et rollback ;
- sous-domaines publics et HTTPS ;
- Kubernetes.

## 6. Architecture du MVP

```text
Navigateur ou client HTTP
           ↓
       API Izyploy
           ↓
  Orchestrateur de déploiement
      ├── client Git
      ├── client Docker
      └── stockage des états et logs
                    ↓
             Docker de l'hôte
              ├── Izyploy
              ├── application A
              └── application B
```

À terme, Izyploy sera lui-même dockerisé. Son worker accédera au moteur Docker de l'hôte par le socket `/var/run/docker.sock`. Les applications créées seront des conteneurs frères, pas des conteneurs imbriqués.

Pendant les premières étapes, l'API pourra tourner directement sur la machine de développement afin de faciliter le débogage. Sa dockerisation sera une étape pédagogique distincte.

## 7. Modèle fonctionnel minimal

Une application possède au minimum :

| Champ | Rôle |
| --- | --- |
| `id` | Identifiant généré par Izyploy |
| `name` | Nom lisible de l'application |
| `git_url` | URL du dépôt GitHub public |
| `branch` | Branche à déployer |
| `container_port` | Port HTTP écouté dans le conteneur |
| `status` | État actuel du déploiement |
| `host_port` | Port attribué sur la machine hôte |
| `url` | URL d'accès calculée |
| `error` | Dernière erreur éventuelle |
| `created_at` | Date de création |
| `updated_at` | Date de dernière modification |

États envisagés :

```text
queued
cloning
building
starting
running
failed
deleting
```

## 8. API envisagée

Cette API est une proposition à valider avant son implémentation.

```text
GET    /health
POST   /applications
GET    /applications
GET    /applications/{id}
GET    /applications/{id}/logs
DELETE /applications/{id}
```

Exemple de création :

```json
{
  "name": "hello-rust",
  "git_url": "https://github.com/example/hello-rust.git",
  "branch": "main",
  "container_port": 8080
}
```

## 9. Contraintes et sécurité

Construire un `Dockerfile` fourni par un tiers et donner accès au socket Docker sont des opérations très privilégiées. Le MVP doit être considéré comme un outil personnel exécutant uniquement des dépôts de confiance.

Les premières protections prévues sont :

- restriction des sources à `github.com` ;
- absence de commandes shell construites par concaténation ;
- noms d'images et de conteneurs générés par la plateforme ;
- limites CPU, mémoire et nombre de processus ;
- étiquette Docker permettant d'identifier les ressources gérées ;
- espace de travail séparé par déploiement ;
- nettoyage des conteneurs et fichiers temporaires ;
- avertissement clair interdisant l'exposition publique à des utilisateurs non fiables.

Docker seul n'est pas une frontière de sécurité suffisante pour exécuter du code hostile. Ce sujet fera l'objet d'une étape de durcissement séparée.

## 10. Critères de réussite du MVP

Le MVP est terminé lorsque :

- un dépôt de test public contenant un `Dockerfile` peut être soumis à l'API ;
- son image est construite sans intervention manuelle ;
- son conteneur démarre et répond depuis le navigateur ;
- les états et erreurs sont consultables ;
- les logs de déploiement sont disponibles ;
- la suppression retire le conteneur et les ressources temporaires ;
- le parcours est documenté et reproductible sur une nouvelle machine équipée de Docker.

## 11. Évolutions après le MVP

L'ordre cible est :

1. reverse proxy et sous-domaines locaux ;
2. déploiement sur VPS, DNS wildcard et HTTPS ;
3. webhooks GitHub et redéploiement ;
4. historique, health checks et rollback ;
5. observabilité et durcissement ;
6. file de travaux et plusieurs workers ;
7. registre d'images ;
8. backend d'exécution Kubernetes.

