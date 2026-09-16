$ErrorActionPreference = 'Stop' #this doesnt actually stop if wasm pack fails lol
wasm-pack build -t web --dev
npx http-server ./ -p 8000

