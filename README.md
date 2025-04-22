
# Url Shortener

A full stack app designed for shortening urls. It's main point is to allow user to create permalinks and QR codes for them. Later user's will be able to easily change the destination link so that QR codes printed on posters/flyers etc. can send users to different places.


## 🛠 Technologies
Rust, Axum, React, Typescript, SQLite, shadcn/ui
## Requirements
Rust & Cargo, PNPM 

## .ENV requirements
Inside the root of the project you'll need to setup an .env file with these variables:
```
URL_SHORTENER_PORT=2222
URL_SHORTENER_ADDRESS=0.0.0.0
```

## Start Dev Server
Clone the project
```bash
  git clone https://github.com/Isembart/url-shortener
```

Go to the project directory

```bash
  cd url-shortener
```

Start Backend

```bash
  cargo run
```

Start Frontend Dev Server

```bash
  cd shorten-link-frontend
  pnpm run dev
```
### Merged frontend and backend
The backend also server files from /url-shortener/public/www directory on default / endpoint. It allows for easier deployment however it's easier to develop with separate frontend dev server. 
In order to use this merged functionality you'll need to manually build the frontend and move built files to url-shortener/public/www directory


## Contributing

Contributions are always welcome!

## Deployment
In order to use deploy.ps1 script you'll need **WSL** installed and have this configuration in the url-shortener/scripts/.env file:
```
PROJECT_ROOT_IN_WSL=
REMOTE_USER=
REMOTE_HOST=
REMOTE_PATH=
SSH_PORT=
```
**This script is used for easier deployment and is not required for local usage.**
