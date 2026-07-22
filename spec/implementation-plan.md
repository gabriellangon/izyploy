# Izyploy — Plan d'apprentissage et d'implémentation

## Roadmap status

Last updated: 2026-07-22

Current milestone: **None — Milestone 5 completed; Milestone 6 not started**

Legend:

- `[x]` completed and explicitly validated;
- `[ ]` not completed;
- **In progress** identifies the only milestone currently active.

- [x] **Milestone 0 — Project framing**
  - [x] Define and validate the product scope.
  - [x] Define the implementation roadmap.
  - [x] Define the Git and project-knowledge conventions.
  - [x] Validate the technical choices needed before application code.
  - [x] Select the structure and name of the trusted public test repository.
- [x] **Milestone 1 — Manual Docker workflow**
  - [x] Validate and merge the external `izyploy-examples` pull request.
  - [x] Prepare a clean local workspace for the example repository.
  - [x] Select `php` as the first application build context.
  - [x] Build its Docker image manually.
  - [x] Start its container with a published host port.
  - [x] Verify `/` and `/health` from the host.
  - [x] Inspect the container logs and metadata.
  - [x] Stop and remove the container and image.
  - [x] Document the complete manual workflow.
- [x] **Milestone 2 — Rust API skeleton**
  - [x] Initialize the Rust project.
  - [x] Add `GET /health`.
  - [x] Organize the initial modules and shared state.
  - [x] Add application and HTTP request logging.
  - [x] Add the first HTTP test.
  - [x] Document and verify the milestone.
- [x] **Milestone 3 — Application model and persistence**
  - [x] Add the `Application` domain model and initial SQLite migration.
  - [x] Initialize the SQLx pool and run migrations at startup.
  - [x] Implement `POST /applications`.
  - [x] Implement the application read routes.
  - [x] Validate creation inputs.
  - [x] Test valid, invalid, and persistent application data.
  - [x] Clarify route ownership with system and feature routers.
  - [x] Document and verify the milestone.
- [x] **Milestone 4 — Background Git clone**
  - [x] Add the `source_ready` state with a forward migration.
  - [x] Persist deployment logs.
  - [x] Launch source preparation in a Tokio background task.
  - [x] Clone Git with structured arguments into an isolated workspace.
  - [x] Confine and validate the build context and root `Dockerfile`.
  - [x] Transition successful work to `source_ready` and failures to `failed`.
  - [x] Test valid, invalid, and non-blocking source preparation.
  - [x] Document and verify the milestone.
- [x] **Milestone 5 — Docker image build**
  - [x] Decide how the single-deployment permit spans clone and image build.
  - [x] Add an injectable Docker CLI abstraction with structured arguments.
  - [x] Generate safe internal image tags and Izyploy labels.
  - [x] Transition `source_ready` applications through `building` to `image_ready`.
  - [x] Persist Docker build output and failure details.
  - [x] Test successful and intentionally failing builds.
  - [x] Document and verify the milestone.
- [ ] **Milestone 6 — Application start and exposure**
- [ ] **Milestone 7 — Logs, deletion, and recovery**
- [ ] **Milestone 8 — Containerize Izyploy**
- [ ] **Milestone 9 — Minimal web interface**
- [ ] **Milestone 10 — Traefik and subdomains**
- [ ] **Milestone 11 — VPS deployment**
- [ ] **Milestone 12 — Redeployment and GitHub webhooks**
- [ ] **Milestone 13 — Observability and hardening**
- [ ] **Milestone 14 — Distributed queue and workers**
- [ ] **Milestone 15 — Kubernetes runtime**

This section is the immediate source of truth for project progress. It must be updated in the same commit that starts or completes a milestone. A milestone can be checked only after its validation criteria have been met and the user has explicitly accepted the result.

## 1. Manière de travailler

Ce projet sera développé comme un parcours guidé, pas comme une livraison automatique complète.

Pour chaque étape :

1. nous définissons ensemble le résultat attendu ;
2. les notions nécessaires sont expliquées avant le code ;
3. nous réalisons une petite modification observable ;
4. nous exécutons une vérification manuelle ou automatisée ;
5. nous faisons un court bilan de ce qui a été appris ;
6. nous nous arrêtons pour valider avant de passer à l'étape suivante.

Le code d'une étape ne doit pas anticiper plusieurs étapes futures. Une solution simple et remplaçable est préférable tant que le concept courant n'est pas maîtrisé.

## 2. Décisions techniques initiales

Les choix suivants sont validés pour le MVP :

- Rust et Axum pour l'API ;
- Docker CLI pour la première intégration, puis éventuellement l'API Docker avec `bollard` ;
- SQLite avec SQLx pour la persistance locale du MVP ;
- tâches Tokio en arrière-plan avant l'introduction d'une vraie file ;
- ports dynamiques avant Traefik et les sous-domaines ;
- exécution locale de l'API avant sa dockerisation ;
- dépôts GitHub publics et de confiance uniquement ;
- `build_context` optionnel avec `.` par défaut et `Dockerfile` fixe à la racine du contexte.

Ces choix sont conçus pour isoler les apprentissages. Leurs limites et les conditions de leur remplacement sont consignées dans `knowledge.md`. Ils pourront évoluer lors d'un milestone ultérieur au moyen d'une nouvelle décision explicite.

## 3. Milestones

### Milestone 0 — Cadrage

Objectif : savoir précisément ce que nous construisons et ce que nous reportons.

Travail :

- relire `product-spec.md` ;
- valider le vocabulaire, le périmètre et les critères de réussite ;
- choisir les décisions techniques encore ouvertes ;
- choisir la structure et le nom d'un petit dépôt public de test.

Le dépôt retenu est `izyploy-examples`. Il contiendra une application par sous-dossier (`java`, `php`, `python` et `rust`), chaque sous-dossier constituant un contexte de build autonome.

Validation : le MVP peut être expliqué en une phrase et son premier scénario peut être décrit sans ambiguïté.

Point d'arrêt : aucune ligne de code applicatif avant cette validation.

### Milestone 1 — Comprendre le parcours Docker manuellement

Objectif : réaliser une fois à la main ce que la plateforme automatisera ensuite.

Notions : contexte de build, image, conteneur, port interne, port hôte, nom, étiquette et nettoyage.

Travail :

- cloner le dépôt de test dans un dossier temporaire ;
- choisir un sous-dossier comme contexte de build ;
- construire son image depuis ce contexte ;
- démarrer le conteneur avec un port publié ;
- vérifier l'application dans le navigateur ;
- consulter ses logs ;
- arrêter et supprimer le conteneur et l'image.

Livrable : une courte procédure reproductible dans [`docs/milestones/milestone-01-manual-docker-workflow.md`](../docs/milestones/milestone-01-manual-docker-workflow.md).

Validation : chaque commande et chaque ressource Docker créée sont comprises.

Point d'arrêt : bilan avant d'automatiser ce flux en Rust.

### Milestone 2 — Créer le squelette Rust

Objectif : disposer d'une API minimale saine, sans Docker ni base de données.

Notions : projet Cargo, runtime Tokio, route Axum, état partagé, sérialisation JSON et gestion d'erreur HTTP.

Travail :

- initialiser le projet Rust ;
- ajouter `GET /health` ;
- organiser les modules principaux ;
- ajouter le logging applicatif ;
- écrire un premier test HTTP.

Livrable : une API qui démarre et répond avec un statut sain, accompagnée du document d'apprentissage [`docs/milestones/milestone-02-rust-api-skeleton.md`](../docs/milestones/milestone-02-rust-api-skeleton.md).

Validation : compilation, tests et appel manuel de `/health`.

Point d'arrêt : revue de la structure avant d'ajouter le métier.

### Milestone 3 — Modéliser une application et persister son état

Objectif : créer et consulter une application sans encore la déployer.

Notions : modèle de domaine, validation des entrées, migrations SQLx et transitions d'état.

Travail :

- créer le modèle `Application` ;
- créer la première migration SQLite ;
- implémenter `POST /applications` ;
- implémenter les routes de lecture ;
- valider l'URL, la branche, le contexte de build et le port ;
- tester les entrées valides et invalides.

Livrable : une application créée avec l'état `queued` survit au redémarrage de l'API, accompagnée du document d'apprentissage [`docs/milestones/milestone-03-application-persistence.md`](../docs/milestones/milestone-03-application-persistence.md).

Validation : tests API et inspection de la base locale.

Point d'arrêt : revue du modèle et des statuts.

### Milestone 4 — Cloner un dépôt en tâche de fond

Objectif : déclencher un travail long sans bloquer la requête HTTP.

Notions : tâche Tokio, cycle de vie asynchrone, dossier temporaire, capture de sortie et propagation d'erreur.

Travail :

- lancer une tâche après la création ;
- passer de `queued` à `cloning` ;
- cloner le dépôt dans un espace dédié ;
- résoudre et confiner le contexte de build dans le dépôt ;
- vérifier le `Dockerfile` à la racine de ce contexte ;
- enregistrer les logs ;
- passer à `failed` avec une erreur utile en cas d'échec.

Livrable : l'API répond rapidement pendant que le clone se poursuit, accompagnée du document d'apprentissage [`docs/milestones/milestone-04-background-git-clone.md`](../docs/milestones/milestone-04-background-git-clone.md).

Validation : tester un dépôt valide, une URL invalide et un dépôt sans `Dockerfile`.

Point d'arrêt : expliquer les limites d'une tâche en mémoire avant de continuer.

### Milestone 5 — Construire l'image Docker

Objectif : transformer le dépôt cloné en image gérée par Izyploy.

Notions : build context, tags, cache, flux de logs, code de sortie et nettoyage en cas d'échec.

Travail :

- générer un tag interne sûr ;
- exécuter le build depuis le contexte sélectionné sans passer par un shell ;
- faire évoluer l'état vers `building` ;
- diffuser ou stocker les logs du build ;
- identifier l'image avec des labels Izyploy.

Livrable : une image Docker identifiable est créée depuis l'API.

Document d'apprentissage : [`docs/milestones/milestone-05-docker-image-build.md`](../docs/milestones/milestone-05-docker-image-build.md).

Validation : vérifier l'image et provoquer volontairement un build en erreur.

Point d'arrêt : revue des risques liés à un `Dockerfile` non fiable.

### Milestone 6 — Démarrer et exposer l'application

Objectif : obtenir le premier parcours vertical complet.

Notions : réseau Docker, publication de port, limites de ressources, variables d'environnement et état du conteneur.

Travail :

- démarrer un conteneur nommé par Izyploy ;
- publier automatiquement le port interne demandé ;
- appliquer des limites CPU, mémoire et processus ;
- fournir `PORT` à l'application ;
- récupérer le port hôte et construire l'URL ;
- passer à `running`.

Livrable : le dépôt soumis devient une application ouvrable dans le navigateur.

Validation : appel HTTP réel et inspection du conteneur.

Point d'arrêt : démonstration du MVP vertical avant toute interface web.

### Milestone 7 — Logs, suppression et récupération après erreur

Objectif : gérer le cycle de vie minimal proprement.

Notions : idempotence, nettoyage, ressources orphelines et cohérence entre Docker et la base.

Travail :

- exposer les logs de déploiement et d'exécution ;
- implémenter la suppression ;
- supprimer le conteneur, l'image et le dossier de travail ;
- traiter les ressources déjà absentes ;
- définir le comportement après redémarrage d'Izyploy.

Livrable : un déploiement peut être diagnostiqué puis entièrement supprimé.

Validation : répéter création et suppression plusieurs fois sans laisser de ressources orphelines.

Point d'arrêt : déclarer ou non le moteur MVP terminé.

### Milestone 8 — Dockeriser Izyploy

Objectif : exécuter la plateforme dans un conteneur qui pilote le Docker de l'hôte.

Notions : Docker-outside-of-Docker, socket Docker, volumes, utilisateur et surface de privilège.

Travail :

- créer le `Dockerfile` d'Izyploy ;
- créer un fichier Compose ;
- monter le socket Docker et les données persistantes ;
- vérifier que les applications sont des conteneurs frères ;
- documenter le risque administratif du socket.

Livrable : Izyploy et les applications déployées fonctionnent sur le même hôte Docker.

Validation : refaire le scénario complet depuis la version conteneurisée.

Point d'arrêt : revue d'architecture avant le reverse proxy.

### Milestone 9 — Ajouter une interface minimale

Objectif : rendre la démonstration visuelle sans construire un frontend complexe.

Travail :

- formulaire de création ;
- liste et statut des applications ;
- affichage des logs ;
- bouton d'ouverture ;
- bouton de suppression.

Livrable : le scénario principal est réalisable depuis un navigateur.

Validation : une personne découvrant le projet peut déployer le dépôt de test sans utiliser de client HTTP.

Point d'arrêt : fin du premier MVP utilisateur.

## 4. Milestones post-MVP

### Milestone 10 — Traefik et sous-domaines

Remplacer les ports visibles par des routes comme `mon-app.localhost`, puis `mon-app.apps.example.com`.

Apprentissages : reverse proxy, réseau partagé, labels Traefik, DNS wildcard et certificats.

### Milestone 11 — Déploiement sur VPS

Installer la plateforme sur une machine distante, configurer le domaine, HTTPS, pare-feu, sauvegardes et redémarrage automatique.

### Milestone 12 — Redéploiement et webhooks

Recevoir un événement GitHub, reconstruire une nouvelle version, effectuer un health check puis basculer le trafic.

### Milestone 13 — Observabilité et durcissement

Ajouter métriques, tableaux de bord, alertes, quotas, scan d'images, politiques réseau et audit des opérations.

### Milestone 14 — File et workers distribués

Introduire PostgreSQL si nécessaire, Redis ou une file dédiée, reprise des travaux, concurrence contrôlée et plusieurs workers.

Ce milestone constitue une refonte interne importante, mais le moteur qui lance les applications reste Docker. Il ne remplace donc pas encore la version Docker du produit.

### Milestone 15 — Kubernetes

Conserver le clone et le build, pousser l'image dans un registre, puis remplacer le démarrage Docker par la création de ressources Kubernetes :

```text
Deployment + Service + Ingress + ConfigMap + Secret
```

Apprentissages : état désiré, contrôleurs, RBAC, namespaces, requests/limits, probes, rolling updates et autoscaling.

Ce milestone est la rupture architecturale principale du projet : le socket Docker, le démarrage direct des conteneurs et leur exposition par ports ou labels Traefik sont remplacés par l'API et les ressources Kubernetes. Il doit commencer sur une branche dédiée à partir de la dernière version Docker stable.

## 5. Proposition pour la première journée

Le but d'une première journée n'est pas de finir tous les milestones.

### Matin

- valider le milestone 0 ;
- exécuter et documenter le milestone 1 ;
- commencer le milestone 2.

### Après-midi

- terminer le milestone 2 ;
- réaliser le milestone 3 ;
- si le rythme d'apprentissage le permet, commencer le clone asynchrone du milestone 4.

Résultat réaliste : une API bien comprise qui accepte et persiste une demande de déploiement, plus un parcours Docker réalisé manuellement. Le premier déploiement entièrement automatisé peut arriver lors de la session suivante sans précipiter les notions.

## 6. Règle de progression

Un milestone est terminé seulement lorsque :

- le comportement fonctionne ;
- les tests pertinents passent ;
- nous savons expliquer les composants ajoutés ;
- la documentation reflète l'état réel ;
- les limites connues sont écrites ;
- le prochain milestone a été explicitement validé.

## 7. Stratégie Git

La branche `main` contient toujours la dernière version stable et démontrable.

Pour les milestones ordinaires, nous utilisons des branches courtes :

```text
feat/milestone-2-api
feat/milestone-3-persistence
feat/milestone-4-git-clone
```

Chaque branche est relue et testée avant d'être fusionnée dans `main`. Les commits restent petits et correspondent autant que possible à une notion apprise.

À la fin des grandes versions, nous conservons un repère immuable avec un tag :

```text
v0.1.0-docker-mvp
v0.2.0-traefik
v0.3.0-vps
```

Avant le milestone 15, nous créons également une branche longue `docker` depuis la dernière version Docker stable. Elle conserve un point de départ visible et éventuellement maintenable.

Le travail Kubernetes commence ensuite sur une branche séparée :

```text
main / docker
      ↓
feat/milestone-15-kubernetes-runtime
```

La branche et le tag Docker garantissent que cette version reste facile à consulter, lancer et comparer. La migration Kubernetes ne doit pas réécrire l'historique Git.

Si l'architecture le permet, nous conserverons aussi les deux moteurs derrière une abstraction commune, par exemple `DockerRuntime` et `KubernetesRuntime`. Ce choix sera étudié à la fin du milestone 14 ; il ne sera pas introduit prématurément dans le MVP.

Les décisions durables, leurs raisons et les conventions communes sont consignées dans `knowledge.md`. Ce fichier doit être relu au début de chaque milestone et mis à jour dès qu'une décision significative est validée.
