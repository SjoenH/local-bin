#!/bin/bash

# 1. Hent parametere
# $1 er første ord etter scriptet (app-navn)
# $2 er andre ord (dockerfile)
APP_NAME=$1
DOCKERFILE=$2

# Sjekk om begge parametere er med
if [ -z "$APP_NAME" ] || [ -z "$DOCKERFILE" ]; then
    echo "❌ Feil: Mangler parametere."
    echo "Bruk: ./deploy.sh <app-navn> <dockerfile>"
    echo "Eks:  ./deploy.sh min-backend Dockerfile.prod"
    exit 1
fi

# Sjekk om Dockerfilen faktisk eksisterer
if [ ! -f "$DOCKERFILE" ]; then
    echo "❌ Feil: Fant ikke filen '$DOCKERFILE'."
    exit 1
fi

# 2. Definer miljø-variabler
USER_NS="${USER}-devns"
IMAGE_TAG="v$(date +%Y%m%d-%H%M%S)"

echo "🔍 Henter informasjon for $APP_NAME..."
IMAGE_REPO=$(kubectl get devimagerepository default -o 'jsonpath={.status.imageRepositoryURL}')
FULL_IMAGE_NAME="${IMAGE_REPO}:${IMAGE_TAG}"

# 3. Docker-steg
echo "🐳 Bygger image for $APP_NAME..."
docker build -f "$DOCKERFILE" -t "$FULL_IMAGE_NAME" .
docker push "$FULL_IMAGE_NAME"

# 4. Generer YAML (vi inkluderer app-navnet i filnavnet for ryddighet)
TEMP_YAML="deploy-${APP_NAME}.yaml"

cat << EOF > "$TEMP_YAML"
apiVersion: shifter.sparebank1.no/v1
kind: Application
metadata:
  name: $APP_NAME
  namespace: $USER_NS
spec:
  image: $FULL_IMAGE_NAME
  env:
    - name: APP_NAME
      value: $APP_NAME
    - name: DEPLOYED_BY
      value: $USER
EOF

echo "📝 Sender $APP_NAME til Kubernetes..."
kubectl apply -f "$TEMP_YAML"

echo "✅ Ferdig! $APP_NAME er deployet."
echo "🔗 https://${USER_NS}.devns.devaws.sparebank1.no"