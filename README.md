# 图床工具客户端

这个工具由rust制成，并且可以支持图片并行上传，甚至可以帮你把别的网站上面的图片给挪动到我们的图床里。
其实使用范围不仅仅是图片，任何类型的文件都是支持的。

## 安装方法

### 1，Release页下载（可能不可靠）

由于电脑上各种安全软件可能会拦截我们这些没有经过合法签名的程序，因此这种小工具想分发起来非常困难。而且由于作者的电脑资源实在有限，不能编译出来所有的平台。如果遇到问题，建议直接看第2点来自行构建（rust有非常成熟的工具链，自己构建不会很麻烦）

### 2，通过cargo install命令安装（需要魔法）

#### 安装rust

首先你需要安装一下rust，它的官方网站在这里`https://rust-lang.org/tools/install/`

这一步可能会非常的慢，建议你使用魔法。

#### 验证是否安装成功

（安装完之后，要重新启动一下你的shell，才能使刚刚的命令生效）

在命令行窗口中，试着执行一下`rustc -V`或者`cargo -h`命令，如果有正确的信息输出，就说明没啥问题了。

#### 执行cargo install

我们的仓库是符合rust的安装标准的，因此，我们直接使用该命令就可以立刻进行安装了

``` sh
cargo install --git https://github.com/czf0613/pic_bed_client
```

其实编译并不需要多久，问题还是需要魔法（你懂的）

#### 找到可执行文件

正常情况下，可执行文件会放在你的用户目录的`.cargo/bin`文件夹下，完整的可执行文件路径应该是`~/.cargo/bin/nas_pic_bed`（把波浪号换成你自己的电脑的用户目录）

此时，你只需要把这个路径配置到typora里面，就大功告成了



## 命令行参数

使用方法非常简单：

``` sh
/path/to/your/exe "https://others.domain.com/path/to/image.jpg" "/path/to/your/image.jpg"
```

可以填入多个参数（用空格隔开，建议用引号包起来避免解析异常），每个参数代表一个文件的路径，这个文件可以是本地磁盘的文件，也可以是网络上的某个link。

默认情况下，可执行文件的名字叫做nas_pic_bed（Windows下会变成nas_pic_bed.exe），这个名字可以随意更改，不会影响功能。

### 限制

1，每个文件不能超过100MB（以后可能会根据服务器压力调整这个值的大小）

2，如果是网络上的链接，由于技术问题，没有办法很准确推测文件的后缀名，如果URL末尾没有明确的后缀名的时候，程序会默认给它设置为`.unknown`



## 配合Typora等工具

如图所示，直接将可执行文件填入typora的custom command中，即可实现自动上传图片到图床

![image-20251025234618316](https://nas.kevinc.ltd:30002/d/public/pic_bed/storage/1c6f8ccea763267866bde2460af12c628869d3a9d59eb61e034169380cf952a7.png?sign=6UD0DN5VUmkVt1LpQKqsrdq00HaYEHKmCqfBUtLAgPM=:0)