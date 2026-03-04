import React from 'react';
import {
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
  Img,
  staticFile,
  Sequence,
  AbsoluteFill,
} from 'remotion';

const FPS = 30;

const IMG_PATHS = {
  compileFlow: 'daily-2026-03-04/compile-flow.png',
  jvmArch: 'daily-2026-03-04/jvm-architecture.png',
  javaJvm: 'daily-2026-03-04/java-jvm.png',
} as const;

const titleStyle: React.CSSProperties = {
  fontSize: 75,
  fontWeight: 900,
  textAlign: 'center',
  color: '#ffffff',
  textShadow: '0 4px 15px rgba(0,0,0,0.6)',
  margin: 0,
};

const highlightStyle: React.CSSProperties = {
  color: '#00e5ff',
};

// ==========================================
// 场景 1：黄金三秒开头 (0-4秒) - 开门见山版
// ==========================================
const IntroScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const scale = spring({ fps, frame, config: { damping: 14 } });
  const opacity = interpolate(frame, [0, 10], [0, 1]);

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: 40 }}>
      {/* 主标题：直接抛出核心疑问 */}
      <h1 style={{ ...titleStyle, transform: `scale(${scale})`, opacity, lineHeight: 1.4 }}>
        <span style={{ color: '#ffd700' }}>Java 程序</span><br />
        到底是怎么运行的？
      </h1>
      
      {/* 副标题卡片：补充痛点和视频价值 */}
      <div
        style={{
          marginTop: 60,
          background: 'rgba(255, 255, 255, 0.08)',
          border: '1px solid rgba(255, 255, 255, 0.2)',
          padding: '25px 45px',
          borderRadius: 16,
          opacity: interpolate(frame, [25, 40], [0, 1]), // 稍晚一点平滑浮现
        }}
      >
        <p style={{ fontSize: 40, color: '#e8e8e8', margin: 0, fontWeight: 'normal', textAlign: 'center', lineHeight: 1.6 }}>
          从敲下 <span style={{ color: '#00f2fe', fontWeight: 'bold' }}>java Main</span> 开始<br />
          一分钟带你梳理底层全流程
        </p>
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 场景 2：编译 (改为完美居中)
// ==========================================
const CompileScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const textY = interpolate(frame, [0, 15], [30, 0], { extrapolateRight: 'clamp' });
  const opacity = interpolate(frame, [0, 15], [0, 1]);
  const imgPop = spring({ fps, frame: frame - 15, config: { damping: 14 } });
  const imgOpacity = interpolate(frame - 15, [0, 10], [0, 1], { extrapolateRight: 'clamp' });

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: 40 }}>
      <h2 style={{ ...titleStyle, transform: `translateY(${textY}px)`, opacity }}>
        1. <span style={highlightStyle}>编译：充当翻译官</span>
      </h2>
      
      <p style={{ fontSize: 40, color: '#e8e8e8', marginTop: 40, opacity, textAlign: 'center', lineHeight: 1.5 }}>
        机器不认识 .java 源文件<br/>
        由 javac 翻译成跨平台的 <strong style={{ color: '#ffd700' }}>.class 字节码</strong>
      </p>

      <div style={{ marginTop: 60, transform: `scale(${imgPop})`, opacity: imgOpacity }}>
        <Img src={staticFile(IMG_PATHS.compileFlow)} style={{ width: 850, borderRadius: 16, boxShadow: '0 8px 20px rgba(0,0,0,0.5)' }} />
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 场景 3：类加载与内存 (改为完美居中)
// ==========================================
const LoadScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const pop1 = spring({ fps, frame: frame - 15, config: { damping: 14 } });
  const pop2 = spring({ fps, frame: frame - 30, config: { damping: 14 } });
  const pop3 = spring({ fps, frame: frame - 45, config: { damping: 14 } });

  const cardStyle: React.CSSProperties = {
    fontSize: 38,
    color: '#e8e8e8',
    background: 'rgba(255,255,255,0.08)',
    border: '1px solid rgba(255,255,255,0.15)',
    padding: '25px 35px',
    borderRadius: 16,
  };

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: 40 }}>
      <h2 style={titleStyle}>
        2. <span style={highlightStyle}>加载：划分内存地盘</span>
      </h2>
      
      <div style={{ marginTop: 50, transform: `scale(${spring({ fps, frame: frame - 10, config: { damping: 14 } })})` }}>
        <Img src={staticFile(IMG_PATHS.jvmArch)} style={{ width: 850, borderRadius: 16, maxHeight: 380, objectFit: 'contain', background: 'rgba(255,255,255,0.02)', padding: 10 }} />
      </div>

      <div style={{ marginTop: 50, width: '85%', display: 'flex', flexDirection: 'column', gap: 20 }}>
        <div style={{ ...cardStyle, transform: `scale(${pop1})` }}>
          📦 <strong style={{ color: '#ffd700' }}>堆 (Heap)</strong>：存放 new 出来的对象实例
        </div>
        <div style={{ ...cardStyle, transform: `scale(${pop2})` }}>
          📚 <strong style={{ color: '#ffd700' }}>元空间</strong>：存放类元数据（JDK 8+移出堆）
        </div>
        <div style={{ ...cardStyle, transform: `scale(${pop3})` }}>
          🎫 <strong style={{ color: '#ffd700' }}>栈 (Stack)</strong>：记录方法调用的压栈出栈
        </div>
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 场景 4：执行 (完美居中)
// ==========================================
const ExecuteScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const titlePop = spring({ fps, frame, config: { damping: 14 } });
  const jitPop = spring({ fps, frame: frame - 30, config: { damping: 14 } });

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: 40 }}>
      <h2 style={{ ...titleStyle, transform: `scale(${titlePop})` }}>
        3. <span style={{ color: '#00f2fe' }}>执行与 JIT 优化</span>
      </h2>
      
      <div style={{ 
        marginTop: 60, 
        padding: '30px 40px', 
        opacity: interpolate(frame, [10, 25], [0, 1]),
      }}>
        <p style={{ fontSize: 40, color: '#e8e8e8', textAlign: 'center', lineHeight: 1.6, margin: 0 }}>
          默认 <strong style={{ color: '#ffd700' }}>解释执行</strong> 逐行翻译，启动快<br />
          但遇到频繁调用的<span style={{ color: '#ff4066', fontWeight: 'bold' }}>「热点代码」</span>时：
        </p>
      </div>

      <div
        style={{
          marginTop: 40,
          background: 'rgba(255, 255, 255, 0.1)',
          borderLeft: '8px solid #00f2fe',
          padding: '30px 50px',
          borderRadius: '8px 16px 16px 8px',
          transform: `scale(${jitPop})`,
        }}
      >
        <span style={{ fontSize: 50, fontWeight: 'bold', color: '#ffffff' }}>
          ⚡ JIT 编译为本地机器码，大幅提速
        </span>
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 场景 5：全流程总结 (改为完美居中)
// ==========================================
const SummaryScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  
  const imgPop = spring({ fps, frame: frame - 10, config: { damping: 14 } });
  const textOpacity = interpolate(frame, [30, 45], [0, 1]);

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: 40 }}>
      <h2 style={titleStyle}>
        4. <span style={highlightStyle}>一句话总结</span>
      </h2>

      <div style={{ marginTop: 60, transform: `scale(${imgPop})` }}>
        <Img src={staticFile(IMG_PATHS.javaJvm)} style={{ width: 850, borderRadius: 16, background: 'rgba(255,255,255,0.05)', padding: 10 }} />
      </div>

      <div style={{ 
        marginTop: 60, 
        opacity: textOpacity, 
        background: 'rgba(255,255,255,0.08)', 
        padding: '30px', 
        borderRadius: 16, 
        width: '85%' 
      }}>
         <p style={{fontSize: 36, color: '#e8e8e8', textAlign: 'center', lineHeight: 1.8, margin: 0}}>
           编译成字节码，加载进 JVM 内存<br/>
           交由执行引擎处理<br/>
           <strong style={{color: '#ffd700', fontSize: 42}}>这就是跨平台与高性能的秘密。</strong>
         </p>
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 主组件
// ==========================================
export const ArticleSlides: React.FC = () => {
  return (
    <AbsoluteFill style={{ background: '#121212', fontFamily: '"PingFang SC", "Microsoft YaHei", sans-serif' }}>
      <Sequence from={0} durationInFrames={4 * FPS}>
        <IntroScene />
      </Sequence>

      <Sequence from={4 * FPS} durationInFrames={7 * FPS}>
        <CompileScene />
      </Sequence>

      <Sequence from={11 * FPS} durationInFrames={9 * FPS}>
        <LoadScene />
      </Sequence>

      <Sequence from={20 * FPS} durationInFrames={8 * FPS}>
        <ExecuteScene />
      </Sequence>
      
      <Sequence from={28 * FPS} durationInFrames={10 * FPS}>
        <SummaryScene />
      </Sequence>
    </AbsoluteFill>
  );
};