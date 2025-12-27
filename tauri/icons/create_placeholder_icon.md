# 创建占位符图标

由于 Tauri Windows 构建需要图标文件，请创建一个最小的 ICO 文件。

## 方法 1: 使用在线工具（推荐）

1. 访问：https://convertio.co/zh/png-ico/
2. 上传任意一个小的 PNG 图片（建议 256x256）
3. 下载为 ICO 格式
4. 保存为 `tauri/icons/icon.ico`

## 方法 2: 使用 PowerShell（如果安装了 ImageMagick）

```powershell
magick convert -size 256x256 xc:#4A90E2 icon.ico
```

## 方法 3: 下载现成的图标

可以从以下网站下载免费的 CSV/文件图标：
- https://www.flaticon.com/
- https://icons8.com/

## 临时解决方案

如果急需构建，可以创建一个最小的有效 ICO 文件。最简单的方法是使用在线转换工具。

