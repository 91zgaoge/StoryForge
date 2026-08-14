# LXGW WenKai（霞鹜文楷）

- 来源 npm：`lxgw-wenkai-webfont@1.7.0`（与历史 CDN 同源，改为本地）
- 许可：SIL Open Font License 1.1（见 OFL.txt）
- 本目录只打 Regular woff2。`font-weight: 400 500` 映射到同一文件，避免浏览器合成伪粗。
- 禁止在 HTML 里再引入 jsDelivr / Google Fonts 作为幕前正文来源。

`lxgw-wenkai-webfont@1.7.0` 包内只有 unicode-range 子集 woff2，没有完整 Regular。完整字形取自官方仓库 `lxgw/LxgwWenKai@v1.250`（与该 npm 包 `VERSION` 文件一致）的 `fonts/TTF/LXGWWenKai-Regular.ttf`，用 fontTools 压成 woff2。
