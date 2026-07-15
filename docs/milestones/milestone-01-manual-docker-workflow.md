# Milestone 01 — Manual Docker workflow

This guide reproduces manually the Docker lifecycle that Izyploy will automate later. It uses the trusted PHP application from `izyploy-examples` and binds the application only to the local machine.

## Prerequisites

- Git and Docker are installed;
- the Docker Engine is running;
- TCP port `8080` is available on `127.0.0.1`.

The commands below use the standard Docker CLI. In the Izyploy development environment, prefix executable commands with `rtk`, for example `rtk docker build ...`. Use `rtk proxy docker logs ...` when unfiltered container logs are needed.

## 1. Prepare the build context

Clone the example repository and enter its PHP build context:

```bash
git clone https://github.com/gabriellangon/izyploy-examples.git
cd izyploy-examples/php
```

The final `.` in the next command refers to this directory. Docker can use its `Dockerfile` and files, but it cannot access sibling application directories as part of this build context.

## 2. Build the image

```bash
docker build --tag izyploy-example-php:milestone-1 .
```

Verify the resulting image:

```bash
docker image inspect \
  --format '{{.Id}} | {{.RepoTags}} | {{.Config.User}} | {{.Config.ExposedPorts}}' \
  izyploy-example-php:milestone-1
```

The image must use the `app` runtime user and expose `8080/tcp`.

## 3. Start the container

```bash
docker run \
  --detach \
  --name izyploy-php-manual \
  --publish 127.0.0.1:8080:8080 \
  izyploy-example-php:milestone-1
```

The port mapping sends traffic from `127.0.0.1:8080` on the host to port `8080` inside the container. Binding to `127.0.0.1` prevents access through other host network interfaces.

Confirm that the container is running:

```bash
docker ps --filter name=izyploy-php-manual
```

## 4. Verify the application

```bash
curl --fail http://127.0.0.1:8080/
curl --fail http://127.0.0.1:8080/health
```

Expected responses:

```json
{"message":"Hello World","runtime":"php"}
```

```json
{"status":"ok"}
```

## 5. Inspect logs and metadata

Read the container logs:

```bash
docker logs izyploy-php-manual
```

The logs must show the PHP server startup and the requests to `/` and `/health`.

Inspect the main runtime properties:

```bash
docker inspect \
  --format 'Image={{.Config.Image}}
State={{.State.Status}}
User={{.Config.User}}
Labels={{json .Config.Labels}}
Ports={{json .NetworkSettings.Ports}}' \
  izyploy-php-manual
```

The container must be `running`, use the expected image and `app` user, and publish `8080/tcp` on `127.0.0.1:8080`. This manual container has no labels; Izyploy will add management labels when it automates resource creation.

## 6. Clean up

Stop and remove the container before removing its image:

```bash
docker stop izyploy-php-manual
docker rm izyploy-php-manual
docker image rm izyploy-example-php:milestone-1
```

Confirm that no resource from this workflow remains:

```bash
docker ps --all --filter name=izyploy-php-manual
docker images izyploy-example-php
```

Both commands must return an empty result table.

## Lifecycle learned

```text
Dockerfile + build context
          ↓ docker build
         image
          ↓ docker run
       container
          ↓ published port
   host HTTP endpoint
          ↓ stop and remove
        no resource
```

The image is the immutable application template. The container is one running instance of that image. Publishing a port makes the container reachable from the host, while explicit cleanup prevents obsolete containers and images from accumulating.
