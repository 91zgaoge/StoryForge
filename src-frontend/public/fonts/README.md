# LXGW WenKai（霞鹜文楷）

- 来源：官方仓库 `lxgw/LxgwWenKai@v1.250` 的 TTF，用 fontTools 压成 woff2。v1.250 的 GitHub Release 资源已空，TTF 从该 tag 的 `fonts/TTF/` 取得（构建时经 jsDelivr 拉取 TTF；运行时幕前不引入 jsDelivr）。
- 许可：SIL Open Font License 1.1（见 OFL.txt）
- Regular：v1.250 `LXGWWenKai-Regular.ttf` → `lxgwwenkai-regular.woff2`（CSS `font-weight: 400`）
- 更重字重：v1.250 没有 Medium TTF，只有 `LXGWWenKai-Bold.ttf`（LXGW 后来把这一面改名为 Medium）。该 TTF → `lxgwwenkai-medium.woff2`，CSS 仍声明 `font-weight: 500`。字体表内 OS/2 `usWeightClass` 为 700、subfamily 为 Bold。
- 禁止把 400 与 500 映射到同一文件。禁止在 HTML 里再引入 jsDelivr / Google Fonts 作为幕前正文来源。
