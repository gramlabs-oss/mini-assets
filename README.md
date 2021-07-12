# mini-assets

提供~~检索、下载~~、生成适用于 [policr-mini](https://github.com/Hentioe/mini-assets) 项目图片验证资源的工具。

## 安装

通过 [Releases](https://github.com/Hentioe/mini-assets/releases) 页面下载预编译的程序，即可直接运行。针对 Linux 发布的预编译版本使用 musl libc 交叉编译，纯静态不挑剔发行版。

您也可以[编译](#从源码编译)生成程序文件，本项目理论支持 Rust 语言所支持的一切硬件架构。

## 使用

当前此项目并未提供对资源的检索和下载相关的功能，这是日后可能实现的计划。也就是说图片仍然需要自己收集，提供的唯一工具 `mini-assets-gen` 程序可以辅助生成清单文件、压缩并规范化图片信息。

### 使用前的准备

收集自己的图片，以类别命名文件夹隔离存放，无需在意图片的大小、命名或格式。

假设我找了几张图，有两张“狗熊”和一张“外星人”以及一张“鲸鱼”。我可以按照以下方式存放它们：

```bash
.
├── 哺乳动物
│   └── jingyu.jpg
├── 狗熊
│   ├── gouxiong1.png
│   └── gouxiong2.jpg
└── 外星人
    └── waixingren.jpeg
```

_注意，这里我为了让阅读本教程的人通过文字辨别图片，使用拼音命名了图片。实际上你无需在意图片的名称。_

至此，我们已经准备好了，可以使用工具了。

### 生成验证资源

验证资源实际上是一种易于理解的简单结构，它由清单文件 `Manifest.yml` 和数个图片文件夹组成。您唯一需要做的，就是在执行某些完命令以后，编辑清单文件，再重新生成。

使用刚才下载的名为 `mini-assets-gen` 的程序，将它放在命令行可以方便执行到的位置。这里的 `*-gen` 名称含义就是“生成器（generator）”。在最初的构思中，资源的检索、下载、生成是分步骤进行的，目前只提供了最后一个步骤的工具，也就是本小节所使用的工具。

执行以下命令：

```bash
./mini-assets-gen --prefix=./botimages
```

对此命令进行一些附加解释：

- 对于 Windows 用户，您执行的文件名是 `mini-asset-gen.exe`。其余的一切，都几乎完全等同（可能存在一些系统差异，例如 Windows 中路径分割符是 `\` 而不是 `/`）。
- 参数 `--prefix` 表示存放图片文件夹们的位置。这里假定所有图片文件夹都放在 `./botimages` 这个路径中。

如果您将 `mini-assets-gen` 和图片文件夹放在同一个根目录中，可以忽略 `--prefix` 参数，因为此参数的默认值就是 `.`（当前目录）。

执行完毕后的文件树：

```bash
.
├── 哺乳动物
│   └── jingyu.jpg
├── 狗熊
│   ├── gouxiong1.png
│   └── gouxiong2.jpg
├── 外星人
│   └── waixingren.jpeg
├── _albums # 输出目录
│   ├── 哺乳动物
│   │   └── e4d3220a2106b04.jpg
│   ├── 狗熊
│   │   ├── b6ea40032672ba3.jpg
│   │   └── f1fc8db0bb6bfa3.png
│   ├── 外星人
│   │   └── 3b5f1e2f96215b6.jpeg
│   └── Manifest.yaml
└── Manifest.yaml # 清单文件
```

您可以注意到以下几点：

1. 在存放图片文件夹的根目录生成了一个名为 `_albums` 的输出目录，此目录内部有和根目录相同的一堆文件夹但文件名不同的图片。
1. 在存放图片文件夹的根目录和 `_albums` 目录内，都生成了一个内容基本等同的 `Manifest.yml` 清单文件。

您需要遵从以下规则：

- 不要对 `_albums` 目录内的任何文件进行变动，包括修改、删除等。
- 修改根目录的 `Manifest.yml` 文件或对根目录的文件夹进行了增减，都需要重新执行生成命令。

## 解析清单文件

我们打开 `Manifest.yml` 会看到以下内容：

```yaml
---
version: 0.1.0 # 生成器版本，无需编辑。
datetime: "2021-07-12T16:22:07Z" # 生成时间，无需编辑。
width: 250 # 生成图片的统一宽度，无需编辑。
include_formats: # 包含的文件格式，无需编辑。
  - jpeg
  - png
  - jpg
albums: # 图集配置，部分可以并且可能需要编辑。
  - id: 外星人 # 图集的 id，无需编辑。
    parents: [] # 图集的父级，可以编辑。
    name: # 图集的国际化名称，可以编辑。
      zh-hans: 外星人 # 图集的中文名。
      zh-hant: ~ # 图集的繁体中文名。
      en: ~ # 图集的英文名。
  - id: 狗熊
    parents: []
    name:
      zh-hans: 狗熊
      zh-hant: ~
      en: ~
  - id: 哺乳动物
    parents: []
    name:
      zh-hans: 哺乳动物
      zh-hant: ~
      en: ~
```

如文件中的注释所述，所有写上“无需编辑”的部分都不要修改。因为即使修改，也会被生成器二次重置。需要编辑的地方，主要是 `albums` 的子项。  
在 `albums` 里边可以看到对应一个个图片文件夹的单独配置，这里的 `id` 就是文件夹的名称。在上文中提到过我们要以类别命名文件夹，但是在清单或验证资源中，这部分叫做 album（图集）。每一个图集默认是一个类别，但是它们并不一定互相独立（后文会解释）。

每一个图集的 `id` 都是生成器在扫描图片目录时自动生成的，它根据图片文件夹的名字命名。您不需要修改 `id` 因为它并不代表“名称”。因为当前 [policr-mini](https://github.com/Hentioe/) 项目暂未支持其它语言，所以假定了文件夹会以中文命名，并且进一步假定生成了 `name.zh-hans` 字段的值（中文名称）。  
您当然可以修改 `name` 部分为任何值，不需要和 `id` 对应。不过，您需要至少填写某一种语言的 `name` 部分，否则将无法生成答案文本。并且，这里建议以某种语言来命名图集文件夹的名称，因为这样容易分辨。假设你把文件夹命名为 `1`、`2`、`3` 之类的数字，仅从 `id` 是看不出含义的。

最后，您还会注意到，每一个图集都有一个 `parents` （父级）配置，并且它们默认都是空的列表。编辑 `parents` 可以在图集之间建立关系，您可以做，也可以不做。

### 编辑清单文件

我们将 `albums` 部分编辑为以下的样子：

```yaml
albums:
  - id: 外星人
    parents: []
    name:
      zh-hans: 外星人
      zh-hant: 外星人
      en: ET
  - id: 狗熊
    parents: [哺乳动物]
    name:
      zh-hans: 狗熊
      zh-hant: 狗熊
      en: black bear
  - id: 哺乳动物
    parents: []
    name:
      zh-hans: 哺乳动物
      zh-hant: 哺乳動物
      en: mammalia
```

重新执行生成命令，再查看清单文件，它被纠正为如下的样子：

```yaml
albums:
  - id: 外星人
    parents: []
    name:
      zh-hans: 外星人
      zh-hant: 外星人
      en: ET
  - id: 狗熊
    parents:
      - 哺乳动物 # 每个父级单独占一行
    name:
      zh-hans: 狗熊
      zh-hant: 狗熊
      en: black bear
  - id: 哺乳动物
    parents: []
    name:
      zh-hans: 哺乳动物
      zh-hant: 哺乳動物
      en: mammalia
```

之所以用「纠正」这个词，而不是格式化，是因为重新执行生成命令确实有纠正作用。例如它会进行以下内容修正：

- 删除所有不存在文件夹的 `album` 配置。例如你在清单文件中添加了一个 `album` 但是并没有为此建立文件夹，那么它将被自动删除。又或者是你删除了某个图集对应的文件夹。
- 删除所有不存在的父级。例如你将「哺乳动物」这个 `id`（父级是其它图集的 id） 写错成了「哺乳动物 1」，那么这个父级会被自动删除。
- 增加清单中不存在但是文件系统中新增的图集。例如你在生成命令结束以后，又增加了一个图集文件夹，那么它会被自动添加成为 `albums` 的一个新的子项目。

此次编辑，将 `alubms` 部分待填充的名称部分填写完整了，并且引用「哺乳动物」这个图集作为「狗熊」的父级。需要注意的是，其实您可以不填充名称的其它语言（保持默认的 `~` 符号），也可以不为任何图集配置父级。也就是说，默认生成出来的清单文件就是可直接使用的。

### 图集之间的关系

如上一小节最后一句话所述，您可以不配置任何图集的父级。但是配置父级可以避免冲突，在某些情况下也许是有必要的。例如本教程的例子中出现的「狗熊」和「哺乳动物」就存在一定的认知重叠。

如果一个验证问题中使用了狗熊的图片，并且恰好又同时生成了「狗熊」和「哺乳动物」两个候选答案。此时，您应该选择狗熊还是哺乳动物呢？严格上来说两个都对，因为问题并没有要求你将物种细分到可选答案中的最小粒度。  
那么问题来了：

1. 图片验证的候选答案个数是比较少的，如果存在多个正确答案，通过率太高了。
1. 没有任何地方有描述过「狗熊」和「哺乳动物」存在关系。

为了避免第一点，为了指明第二点，为这类图集添加父级是有必要的。这就是配置图集的 `parents` 字段的理由。

实际上生成器仅仅只是建立了图集关系，具体如何生成验证数据，和资源生成器无关。我们将「狗熊」和「哺乳动物」建立关系，至于如何处理这种关系，是 [policr-mini](https://github.com/Hentioe/mini-assets) 项目的责任。这部分的细节日后可能会在该项目中单独描述。

**错误的图集关系**

让图集之间互相引用是错误的，这包括直接或间接的互相引用。例如给「狗熊」添加「哺乳动物」这个父级，又给「哺乳动物」添加「狗熊」作为父级。先不提这在逻辑上就是不成立的，更重要的是会造成验证资源使用方的“死循环解析”，具体表现为机器人在启动时陷入僵死的状态。虽然生成器或机器人（或其它使用方）可以在生成或解析的过程中检查出这种错误，但是不表示它们知道如何处理这种错误。

间接的互相引用例如「生物」是「哺乳动物」的父级，「哺乳动物」是「狗熊」的父级，但是「狗熊」又被添加成了「生物」的父级。虽然狗熊的直接父级并非生物，但是间接父级是生物，所以这可能导致和直接的互相引用没有什么区别的错误。

**危险的图集关系**

上文有解释过建立图集的原因，如果看明白了就会知道，本教程并不建议可以的去建立它们之间的关系。例如你找了某种动物的图片，又专门建立这种动物的某个科某个属的图集，然后再塞入一些比较容易和这种动物产生混淆的图片。说实话，这样做没有意义。

假设「狗熊」是一个图集，「老虎」是一个图集。然后又专门建立了一个「动物」图集，并把所有具体动物的图集们都添加上「动物」这个父级。这意味着什么？这意味这每生成一次验证数据，都只能在所有的动物中选择一张图片。假设你有两大类的图集，分别是「动物」和「汽车」，这两个图集被无数子图集引用作为父级。即时你准备有几千几万张图片，也只能生成至多两个答案的验证数据。为什么呢？

因为当第一张图集资源被决定的时候，它一定是动物或汽车二者之一。当它是动物时，就不会继续在动物下的所有图集中找资源了，只能是某种汽车了。这将导致验证数据的生成过于单调，甚至满足不了指定的数量需求（例如这里，至多只能生成 2 个答案，无论你有多少个图集。因为它们被人为的分成了 2 个大类）。

**一个恰当的例子**

假设你要制作一套以动漫为主题的验证资源，你可以用以下的方式组织单个动漫的资源：

```
- 某动漫A（图集）
  - 动漫A主角1（图集）
    - 动物A主角1_1（图片）
    - 动物A主角1_2（图片）
  - 动漫A主角2（图集）
    - 动物A主角2_1（图片）
    - 动物A主角2_2（图片）
  - 动漫A配角1（图片）
  - 动漫A配角2（图片）
```

你可以注意到动漫 A 的主角、配角都属于动漫 A。但主角是作为独立的图集，而配角是直接放在父级下的图片。这样在显示知名度更高的主角时以主角自己的名称，而知名度低的配角时以动漫名作为名称。然后每一个动漫只有这一层简单关系，其余各自独立。这样的话既能保证数据不再单一，又能保证不会冲突（验证中出现同一个动漫的多个人物作为候选答案）。

简单的说，危险的图集关系就是刻意建立“大”的分门别类。

## 附加的命令参数

您可能有注意到清单文件中的宽度值为 `250`（像素），这是默认值。一般来讲，不建议将宽度调整太大，应该尽可能的小。因为图片越大加载越慢，会增加网速慢的用户验证超时的可能性。

通过 `--width=<value>` 参数调整图片宽度，例如：

```
./mini-assets-gen --width=180 --prefix=./botimages
```

执行完这条命令，清单文件中的 `width` 的值会自动更正为 `180`，并且所有图片会按照这个宽度等比缩放。

如果只是单纯的调整宽度，您无需删除已有的 \_albums 文件夹（输出目录）。因为 \_albums 里边的所有图片是以原图的摘要信息命名的，只要原图没有变，无论怎样压缩，输出后的文件名都是一样的。举个例子，原图 `a.jpg` 的摘要是 `b6ea40032672ba3`，那么它无论被怎样的参数压缩多少遍，生成的图片都是以 `b6ea40032672ba3.jpg` 来命名的。因为同名覆盖的原理，您无需删除上一次的输出目录。

但是，假设您删除了某些图片，您需要删除整个 \_albums 目录。因为当前生成器并不会帮你跟踪图片文件的删除，如果不这样做，会导致上一次的输出结果和这次的共存。

## 安装验证资源

将输出目录 \_albums 压缩成一个 zip 文件，一般来讲叫 \_albums.zip。

打开您自行部署的 [policr-mini](https://github.com/Hentioe/mini-assets) 实例的后台，在系统菜单中找到「全局属性」页面中的「更新图片验证的资源」部分，通过「选择资源包」按钮选中本地 \_albums.zip 文件并完成上传。

_如果您部署的实例的后台没有这部分内容，请更新到最新的版本上。_

确保页面没有响应你一个错误提示（因为上传完毕后台会解析这个 zip 文件，检查结构是否正确），点击「确认更新」按钮即可完成验证资源的更新。

_注意，当前 policr-mini 项目仅支持 zip 格式。请不要生成其它格式压缩包或者加密压缩。_

## 从源码编译

安装 Rust 工具链，使用如下命令即可完成程序构建：

```bash
cargo build --release
```

本项目支持著名的图像处理软件 ImageMagick 作为图像处理后端，它可以带来更优质的算法以及更快的速度。

您需要安装 magickwand 库，在 Debian 系的系统中包名一般叫 `libmagickwand-dev`。使用如下命令生成以 ImageMagick 作为图像处理后端的程序：

```bash
cargo build --release --no-default-features --features=magickwand
```

如果您是 NixOS 用户，本项目提供了 nix-shell 的配置文件。通过以下命令可直接完成构建：

```bash
nix-shell .nix/magickwand.nix \
  --command 'cargo build --release --no-default-features --features=magickwand'
```

## 结束语

如果您有任何疑惑，或者对细节的追究，都可以在 issues 中提出。如果您有什么好的想法，不排除被本项目或 polcir-mini 项目采纳的可能性。
