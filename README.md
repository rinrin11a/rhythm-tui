# rhythm-tui

一个用 Rust 写的终端音游：4 轨下落，配合程序实时合成的节拍音乐。

## 特点

- 音乐完全由程序合成（底鼓、踩镲、低音、和弦），不需要音频文件，也没有版权问题。
- 谱面随机生成，和音乐节拍对齐（120 BPM，约 64 秒，第一小节为前奏）。
- 判定窗口：PERFECT ±40ms、GREAT ±80ms、GOOD ±120ms，超出即 MISS。
- 连击、分数（每 10 连击倍率 +1，最高 ×4）、准确率与评级（S / A / B / C / D）。
- 延迟校准：音频设备有延迟时，可以用 `[` `]` 微调。
- 没有音频设备时自动静音运行。

## 运行

需要 Rust stable 工具链。Linux 上还需要 ALSA 开发库（Debian / Ubuntu：`sudo apt install libasound2-dev`）。

```bash
cargo run --release
```

## 操作

| 按键 | 功能 |
| --- | --- |
| `D` `F` `J` `K` | 打击对应轨道 |
| `[` / `]` | 延迟校准 -5ms / +5ms |
| `R` | 重新开始 |
| `Q` / `Esc` | 退出 |

判定提示里的数值（如 `GREAT +35ms`）为正表示偏晚，为负表示偏早。如果平均偏差一直是正的，按 `]` 增加校准值；偏早则按 `[`。

## 项目结构

```
src/
├── main.rs   主循环、键盘输入、开始与重开
├── chart.rs  谱面生成（随机种子）与随机数
├── audio.rs  音乐与打击音合成、音频播放
├── game.rs   判定窗口、计分、连击、评级
└── ui.rs     ratatui 绘制：判定线、音符、状态栏、结算面板
```

## 测试

```bash
cargo test
cargo clippy --all-targets
```
