# Mailwoman Helm chart

A hardened Kubernetes deployment of the Mailwoman server.

## Install

```sh
# Build + push the hardened FROM-scratch image first (see the repo Dockerfile):
docker build --target runtime-static -t <registry>/mailwoman:26.19.0 .
docker push <registry>/mailwoman:26.19.0

# Seal-at-rest key (required for data to survive restarts):
kubectl create secret generic mailwoman-key \
  --from-literal=MW_SERVER_KEY=$(openssl rand -hex 32)

helm install mailwoman packaging/helm/mailwoman \
  --set image.repository=<registry>/mailwoman \
  --set image.tag=26.19.0 \
  --set serverKeySecret.name=mailwoman-key
```

## Security posture (defaults)

The chart does not create or default an administrator identity. For a new account,
choose a readable name once, for example `admin-$(openssl rand -hex 6)`, and persist
that value in your deployment configuration or existing credential Secret before
installing. Supply the same `MW_ADMIN_USER` on subsequent releases; keep the
password in your organization's secret-management system. Do not generate a new
name inside a Helm template on every render or upgrade. Existing explicit names
remain valid and must not be renamed as part of an upgrade.

The pod ships locked down out of the box:

- runs as non-root uid/gid 65532; `runAsNonRoot: true`
- read-only root filesystem (only the `/data` PVC and an in-memory `/tmp` are writable)
- all Linux capabilities dropped
- `allowPrivilegeEscalation: false`, not privileged
- `seccompProfile: RuntimeDefault`
- service-account token not mounted (the app never calls the Kubernetes API)

These map to the same confinement applied by the container overlay
(`docker-compose.hardened.yml`) and the systemd unit (`docs/deploy/mailwoman.service`).

## Validate locally

```sh
helm lint packaging/helm/mailwoman
helm template mailwoman packaging/helm/mailwoman
```

Both run in CI (`.github/workflows/supply-chain.yml`, `helm-lint` job).
