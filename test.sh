#!/bin/bash

# 创建translated文件夹（如果不存在）
mkdir -p test/translated

# 遍历test文件夹下的所有.c文件
for input_file in test/*.c; do
    # 获取文件名（不带路径和扩展名）
    base_name=$(basename "$input_file" .c)
    
    # 设置输出文件路径
    output_file="test/translated/${base_name}.dfy"
    
    # 运行cargo命令进行转换
    ./target/debug/CParser.exe -i "$input_file" -o "$output_file"
    
    # 打印处理信息
    echo "Processed $input_file -> $output_file"
done

echo "All files have been processed."