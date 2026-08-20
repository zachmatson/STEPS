#! /bin/sh

docker run --rm \
  --volume "$PWD/JOSS":/data \
  --user $(id -u):$(id -g) \
  --env JOURNAL=joss \
  openjournals/inara

rm -r ./JOSS/jats
