# Rust 调用大漠插件.

Rust 调用 Python 调用大漠插件, 然后将大漠插件的执行结果返回给 Rust.


## 使用

> [!TIP]
> 
> 需要安装 Python 32 位版本.

将大模的 dll 放到 py_dm 目录中, 然后编译为 exe.

```shell
# pip install pyinstaller

pyinstaller --onefile --console --clean --hidden-import=DMInstanceManager --add-data "DmReg.dll;." --add-data "dm.dll;." --name dm_worker worker.py
```

将编译好的 exe 与 Rust 主程序放到同一个目录下.
