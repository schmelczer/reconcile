#!/bin/bash

cd backend
cargo set-version --bump patch
cd ../plugin
npm version patch
git add .
git commit -m "Bump versions"
TAG=$(node -p "require('./package.json').version")
git tag -a $TAG -m "Release $TAG"
git push origin $TAG
