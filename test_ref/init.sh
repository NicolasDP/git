#! /bin/bash

git init

git config user.email "git-test@example.com"
git config user.name  "Test"

echo "README" > README.md
git add README.md
git commit -m "initial commit"

git remote add origin https://github.com/NicolasDP/git
git fetch
