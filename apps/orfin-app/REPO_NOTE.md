# Nota de publicação

Este client foi publicado em `orfin-rust/apps/orfin-app` porque o token do Cloud Agent
(GitHub App) **não tem permissão** `createRepository`.

Para extrair para um repo próprio depois:

```bash
git subtree split -P apps/orfin-app -b orfin-app-split
# num repo vazio edevPedro/orfin-app:
git push git@github.com:edevPedro/orfin-app.git orfin-app-split:main
```
