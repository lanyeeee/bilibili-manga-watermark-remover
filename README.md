# 哔哩哔哩漫画去水印工具

bilibili漫画 哔哩哔哩漫画 B漫 去水印工具，带图形界面，带下载功能，图形界面基于[Tauri](https://v2.tauri.app/start/)

在[Release页面](https://github.com/lanyeeee/bilibili-manga-watermark-remover/releases)可以直接下载

# 效果预览

| 原图                                 | 去水印                                 |
| ------------------------------------ | -------------------------------------- |
| <img src="md/少女终末旅行-原图.jpg"> | <img src="md/少女终末旅行-去水印.jpg"> |
| <img src="md/炎拳-原图.jpg">         | <img src="md/炎拳-去水印.jpg">         |

# 使用方法

### 去水印

1. 选择漫画目录，等待自动生成背景水印图完成
2. 点击开始去水印按钮，等待去水印完成
3. 前往输出目录查看结果

下面的视频是去水印的完整流程

https://github.com/user-attachments/assets/f7ad65d0-4211-4fe3-b090-419a722b2e45


### 生成背景水印图

一般选择漫画目录后，工具会自动为每种尺寸的图片生成黑色和白色的背景水印图 
如果自动生成失败，可以尝试手动截取水印

下面的视频演示了

- 所有尺寸全部重试自动生成
- 单个尺寸重试自动生成
- 单个尺寸手动截取水印

https://github.com/user-attachments/assets/52666942-27df-4e39-9dc1-dfcbe0461c44

### 下载

下面的视频演示了漫画下载的基本功能

- 漫画搜索
- 扫码登录
- Cookie有效性检测
- 框选、全选
- 漫画下载

https://github.com/user-attachments/assets/dc9bbcc5-28e5-4f48-bb19-fc74ada758e6

# 常见问题

- [生成背景水印图失败](https://github.com/lanyeeee/bilibili-manga-watermark-remover/discussions/1)
- [极个别图片去除水印失败](https://github.com/lanyeeee/bilibili-manga-watermark-remover/discussions/5)
- [同一本漫画，正文尺寸相同，但是水印不同](https://github.com/lanyeeee/bilibili-manga-watermark-remover/discussions/8)

# 去水印原理

本工具的去水印算法基于[这个项目](https://github.com/yuchenxi2000/bilibili-watermark/tree/master)

> B漫给图片添加水印的算法是用一张带alpha通道的水印图叠加到原图上。  
> out = in * alpha + watermark * (1 - alpha)  
> out是加了水印的图，in是原图，alpha是透明通道，watermark是水印（除透明通道外）

因为网上下载的图没有alpha通道，所以需要一张黑背景和一张白背景的水印图把alpha通道算出来  
所以每种尺寸的图片要去水印，都需要对应尺寸的黑背景和白背景水印图各一张  

[核心算法的Python实现](https://github.com/yuchenxi2000/bilibili-watermark/tree/master/B%E6%BC%AB)

# 关于被杀毒软件误判为病毒

对于个人开发者来说，这个问题几乎是无解的(~~需要数字证书给软件签名，甚至给杀毒软件交保护费~~)  
我能想到的解决办法只有：
1. 根据下面的**如何构建(build)**，自行编译
2. 希望你相信我的承诺，我承诺你在[Release页面](https://github.com/lanyeeee/bilibili-manga-watermark-remover/releases)下载到的所有东西都是安全的

# 如何构建(build)

构建非常简单，一共就3条命令  
~~前提是你已经安装了Rust、Node、pnpm~~

#### 前提

- [Rust](https://www.rust-lang.org/tools/install)
- [Node](https://nodejs.org/en)
- [pnpm](https://pnpm.io/installation)

#### 步骤

#### 1. 克隆本仓库

```
git clone https://github.com/lanyeeee/bilibili-manga-watermark-remover.git
```

#### 2.安装依赖

```
cd bilibili-manga-watermark-remover
pnpm install
```

#### 3.构建(build)

```
pnpm tauri build
```
# 免责声明
- 本工具仅作学习、研究、交流使用，使用本工具的用户应自行承担风险
- 作者不对使用本工具导致的任何损失、法律纠纷或其他后果负责
- 作者不对用户使用本工具的行为负责，包括但不限于用户违反法律或任何第三方权益的行为
# 其他

任何使用中遇到的问题、任何希望添加的功能，都欢迎提交issue交流，我会尽力解决

