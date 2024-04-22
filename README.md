# StarryOS

## 模块化介绍

与主仓库的starry相比，本仓库已经实现了所有模块的拆分，除了必要的工具和C语言库和唯一用来运行的helloworld外，所有rust形式的cargo包均已经被上传到https://github.com/Arceos-crates组织中，对各个crates的调用也不再是通过路径调用而是在cargo.toml中通过git从云端获取。目前该仓库仅支持Unikernel模式下的helloworld测例。

启动命令：

```shell
# 构建镜像
./build_img.sh sdcard
# 运行 Unikernel 架构内核
make run
```

获取crates之间的调用图：

```shell
cargo depgraph --workspace-only | dot -Tpng > graph.png
```

通过此命令，调用图会保存在Starry/graph.png中。



## 简介

这里是StarryOS，一个基于ArceOS实现的宏内核。

> Starry意指布满星星的，寓意本OS的开发学习借鉴了许多前辈的思路，并将其汇总归一为这个内核。

在线文档详见：[Starry (azure-stars.github.io)](https://azure-stars.github.io/Starry/)

## 成员

陈嘉钰、郑友捷、王昱栋

## Usage

```shell
# 构建镜像
./build_img.sh sdcard
# 运行 Unikernel 架构内核
make run

# 以宏内核形式启动(当前仅支持 riscv 架构)
make A=apps/oscomp ARCH=riscv64 run

# 使用 ramdisk 加载测例并且运行内核，可以显著提高文件 IO 速度
make A=apps/oscomp ARCH=riscv64 FEATURES=img run

```

## CI 说明
本项目的 CI 结构继承了 arceos 的 CI 结构，同时在其上加入了宏内核的测试用例，以求保证该项目可以在宏内核和 Unikernel 架构以及不同的指令集架构下正常运行。

当前的 Unikernel 基本适配了 arceos 测例，而宏内核仅支持在 riscv 架构上运行。各个 CI 含义如下：

* Clippy CI：代码风格检查

* Test CI / unit-test：单元测试，当前由于宏内核代码紧耦合 riscv 而导致无法通过单元测试

* build CI / build：默认架构 ( Unikernel + x86_64 ) 构建测试

* build CI / build-apps-for-unikernel + ARCH：Unikernel 架构下不同指令集的测例构建测试

* build CI / build-apps-for-monolithic + ARCH：宏内核架构下不同指令集的测例构建测试

* Test CI / app-test-for-unikernel + ARCH：Unikernel 架构下不同指令集的测例运行测试

* Test CI / app-test-for-monolithic + ARCH：宏内核架构下不同指令集的测例运行测试

## 项目结构

### 整体结构图

![image-20230603005345201](https://raw.githubusercontent.com/Azure-stars/Figure-Bed/main/image-20230603005345201.png)



### 模块依赖图

```mermaid
graph TD;
axsync-->axdisplay
axdriver-->axdisplay

axhal-->axdriver
axalloc-->axdriver
axconfig-->axdriver

axdriver-->axfs
axsync-->axfs
axtask-.dev.->axfs

axconfig-->axhal
axalloc-->axhal
axlog-->axhal

axhal-->axnet
axsync-->axnet
axtask-->axnet
axdriver-->axnet

axalloc-->axruntime
axconfig-->axruntime
axdriver-->axruntime
axhal-->axruntime
axlog-->axruntime
axnet-->axruntime
axdisplay-->axruntime
axtask-->axruntime
axprocess-->axruntime
axtask-->axsync
axtask-->axprocess
axfs-->axprocess
axhal-->axprocess

axalloc-->axtask
axhal-->axtask
axconfig-->axtask
axlog-->axtask

axfs-->axmem
axalloc-->axmem
axhal-->axmem
axmem-->axprocess
```

* crates：与OS设计无关的公共组件
* modules：与OS设计更加耦合的组件
* doc：每周汇报文档，当前位于doc分支上
* apps：unikernel架构下的用户程序，继承原有ArceOS
* scripts：makefile脚本，继承原有ArceOS
* ulib：用户库，继承原有ArceOS



## 测例切换和执行

执行如下指令可以生成sdcard文件镜像

```shell
$ ./build_img.sh sdcard
```

如果想要切换到其他测例，如切换到gcc，请在保证testcases/gcc文件夹下对应文件夹内容满足需求之后，执行如下指令

```shell
$ ./build_img.sh gcc
```

当使用 gcc 测例时，由于 gcc 测例内容过大，不直接拷贝到 ramdisk 上，因此不能启动 `FEATURES=img`。

通过修改指令可以切换生成的文件镜像中包含的测例。相应测例存放在`testcases/`文件夹下，如执行`./build_img.sh libc-static`可以生成libc静态测例。



## 文档

内核文档存放在`doc/Starry决赛设计文档.pdf`。

另外，可以通过静态部署网页[Starry (azure-stars.github.io)](https://azure-stars.github.io/Starry/)查看更好排版的文档。