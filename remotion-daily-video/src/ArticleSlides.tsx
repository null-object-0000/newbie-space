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

// 通用大字号样式，专为竖屏短视频设计
const titleStyle: React.CSSProperties = {
  fontSize: 80,
  fontWeight: 900,
  textAlign: 'center',
  color: '#fff',
  textShadow: '0 4px 12px rgba(0,0,0,0.5)',
  margin: 0,
};

const highlightStyle: React.CSSProperties = {
  color: '#00e5ff', // 亮青色，吸引眼球
  fontSize: 90,
};

// ==========================================
// 场景 1：黄金三秒开头 (0-4秒)
// ==========================================
const IntroScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // 弹簧动画：文字“蹦”出来
  const scale = spring({ fps, frame, config: { damping: 12 } });
  const opacity = interpolate(frame, [0, 10], [0, 1]);

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: 40 }}>
      <h1 style={{ ...titleStyle, transform: `scale(${scale})`, opacity }}>
        敲下 <span style={{ color: '#ffb300' }}>java Main</span>
        <br />
        到底发生了什么？
      </h1>
      <p
        style={{
          fontSize: 48,
          color: '#e8e8e8',
          marginTop: 60,
          opacity: interpolate(frame, [30, 45], [0, 1]), // 延迟 1 秒出现
          fontWeight: 'bold',
        }}
      >
        一分钟带你彻底搞懂底原理！
      </p>
    </AbsoluteFill>
  );
};

// ==========================================
// 场景 2：第一步 编译 (4-11秒)
// ==========================================
const CompileScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const textY = interpolate(frame, [0, 15], [50, 0], { extrapolateRight: 'clamp' });
  const opacity = interpolate(frame, [0, 15], [0, 1]);
  const imgScale = spring({ fps, frame: frame - 20, config: { damping: 14 } }); // 图片延迟弹出

  return (
    <AbsoluteFill style={{ justifyContent: 'flex-start', alignItems: 'center', paddingTop: 150 }}>
      <h2 style={{ ...titleStyle, transform: `translateY(${textY}px)`, opacity }}>
        第一步：<span style={highlightStyle}>编译！</span>
      </h2>
      <p style={{ fontSize: 40, color: '#ccc', marginTop: 30, opacity }}>
        源码太难懂，翻译成 <strong style={{ color: '#fff' }}>字节码</strong> (.class)
      </p>

      <div style={{ marginTop: 80, transform: `scale(${imgScale})`, opacity: imgScale }}>
        <Img src={staticFile(IMG_PATHS.compileFlow)} style={{ width: 900, borderRadius: 20 }} />
      </div>

      <div
        style={{
          marginTop: 80,
          background: '#ff2a55', // 抖音红
          padding: '20px 40px',
          borderRadius: 20,
          transform: `scale(${spring({ fps, frame: frame - 60 })})`, // 再次延迟弹出核心考点
        }}
      >
        <span style={{ fontSize: 60, fontWeight: 'bold', color: 'white' }}>
          一次编写，到处运行！
        </span>
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 场景 3：第二步 类加载与内存 (11-20秒)
// ==========================================
const LoadScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // 列表项逐个蹦出
  const pop1 = spring({ fps, frame: frame - 20 });
  const pop2 = spring({ fps, frame: frame - 40 });
  const pop3 = spring({ fps, frame: frame - 60 });

  return (
    <AbsoluteFill style={{ justifyContent: 'flex-start', alignItems: 'center', paddingTop: 150 }}>
      <h2 style={titleStyle}>
        第二步：<span style={highlightStyle}>类加载与划分地盘</span>
      </h2>
      
      <div style={{ marginTop: 60, transform: `scale(${spring({ fps, frame: frame - 10 })})` }}>
        <Img src={staticFile(IMG_PATHS.jvmArch)} style={{ width: 900, borderRadius: 20, maxHeight: 500, objectFit: 'contain' }} />
      </div>

      <div style={{ marginTop: 60, width: '80%', display: 'flex', flexDirection: 'column', gap: 30 }}>
        <div style={{ fontSize: 50, background: 'rgba(255,255,255,0.1)', padding: 30, borderRadius: 16, transform: `scale(${pop1})` }}>
          📦 <strong>堆 (Heap)</strong>：存放你的对象实例
        </div>
        <div style={{ fontSize: 50, background: 'rgba(255,255,255,0.1)', padding: 30, borderRadius: 16, transform: `scale(${pop2})` }}>
          📚 <strong>方法区/元空间</strong>：存放类的信息
        </div>
        <div style={{ fontSize: 50, background: 'rgba(255,255,255,0.1)', padding: 30, borderRadius: 16, transform: `scale(${pop3})` }}>
          🎫 <strong>栈 (Stack)</strong>：发给线程的工作牌(栈帧)
        </div>
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 场景 4：第三步 执行狂飙 (20-28秒)
// ==========================================
const ExecuteScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const titlePop = spring({ fps, frame });
  const jitPop = spring({ fps, frame: frame - 45, config: { damping: 10 } }); // 更猛烈的弹起

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: 40 }}>
      <h2 style={{ ...titleStyle, transform: `scale(${titlePop})` }}>
        最后一步：<span style={{ color: '#00f2fe', fontSize: 100 }}>执行！</span>
      </h2>
      
      <p style={{ fontSize: 50, marginTop: 60, textAlign: 'center', lineHeight: 1.5, opacity: interpolate(frame, [15, 30], [0, 1]) }}>
        逐行 <strong>解释执行</strong> 慢慢跑...
        <br />
        遇到疯狂循环的<span style={{ color: '#ffb300' }}>「热点代码」</span>怎么办？
      </p>

      <div
        style={{
          marginTop: 80,
          background: 'linear-gradient(90deg, #ff0844 0%, #ffb199 100%)',
          padding: '40px 60px',
          borderRadius: 30,
          transform: `scale(${jitPop})`,
          boxShadow: '0 20px 50px rgba(255, 8, 68, 0.4)',
        }}
      >
        <span style={{ fontSize: 80, fontWeight: 'bold', color: 'white' }}>
          🚀 JIT 编译介入，效率狂飙！
        </span>
      </div>
    </AbsoluteFill>
  );
};

// ==========================================
// 主组件：通过 Sequence 拼接时间轴
// ==========================================
export const ArticleSlides: React.FC = () => {
  return (
    <AbsoluteFill style={{ background: '#0f0f13', fontFamily: 'sans-serif' }}>
      {/* 0 - 4 秒：开场悬念 */}
      <Sequence from={0} durationInFrames={4 * FPS}>
        <IntroScene />
      </Sequence>

      {/* 4 - 11 秒：编译阶段 */}
      <Sequence from={4 * FPS} durationInFrames={7 * FPS}>
        <CompileScene />
      </Sequence>

      {/* 11 - 20 秒：加载阶段 */}
      <Sequence from={11 * FPS} durationInFrames={9 * FPS}>
        <LoadScene />
      </Sequence>

      {/* 20 - 28 秒：执行阶段 */}
      <Sequence from={20 * FPS} durationInFrames={8 * FPS}>
        <ExecuteScene />
      </Sequence>
      
      {/* 28 秒之后：可以在这里加一个引导点赞关注的结尾 Sequence，这里暂时留白，画面会停留在狂飙阶段或者你自行补充 */}
    </AbsoluteFill>
  );
};