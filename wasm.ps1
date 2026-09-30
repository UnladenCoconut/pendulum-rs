$ErrorActionPreference = 'Stop' #this doesnt actually stop if wasm pack fails lol
wasm-pack build -t web --dev -d ./site/pkg
npx http-server ./site -p 8000

