#!/bin/bash

SERVICE_NAME=demo-api-v2

docker build -t us-west1-docker.pkg.dev/bokoup/demo/$SERVICE_NAME .
docker push us-west1-docker.pkg.dev/bokoup/demo/$SERVICE_NAME
gcloud beta run deploy $SERVICE_NAME --image us-west1-docker.pkg.dev/bokoup/demo/$SERVICE_NAME --platform managed --region us-west1 --allow-unauthenticated \
--min-instances 1 \
--update-env-vars RUST_LOG=DEBUG \
--service-account demo-bokoup@bokoup.iam.gserviceaccount.com \


# --update-secrets /keys/platform_signer/platform_signer-keypair.json=PLATFORM_SIGNER_KEYPAIR:1
# moved PLATFORM_SIGNER_KEYPAIR to environment variable