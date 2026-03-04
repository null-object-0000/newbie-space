import React from 'react';
import { useCurrentFrame } from 'remotion';

const FPS = 30;
const SEC_PER_SLIDE = 8;
const FRAMES_PER_SLIDE = FPS * SEC_PER_SLIDE;

const bodyStyle = { fontSize: 22, lineHeight: 1.6, maxWidth: 920, marginTop: 16, textAlign: 'left' as const };
const bodyStyleSmall = { fontSize: 20, lineHeight: 1.55, maxWidth: 920, marginTop: 12, textAlign: 'left' as const };

const SLIDES: { title: string; body: React.ReactNode }[] = [
  {
    title: 'Java 代码是怎么运行起来的？',
    body: (
      <>
        <p style={{ fontSize: 26, opacity: 0.9, marginTop: 16 }}>
          从你敲下 <code style={{ background: 'rgba(255,255,255,0.15)', padding: '2px 6px', borderRadius: 4 }}>java Main</code> 到程序输出结果，中间经历了什么？
        </p>
        <p style={{ fontSize: 22, marginTop: 20, color: 'rgba(232,232,232,0.9)' }}>
          Java 代码的运行主要经历 <strong>编译、类加载、执行</strong> 三个阶段。
        </p>
      </>
    ),
  },
  {
    title: '一句话概括',
    body: (
      <p style={{ fontSize: 24, lineHeight: 1.65, maxWidth: 920, marginTop: 24 }}>
        <strong>源码（.java）→ 编译成字节码（.class）→ JVM 类加载进内存 → 解释或 JIT 编译成机器码 → 在 CPU 上执行。</strong>
      </p>
    ),
  },
  {
    title: '1. 编译',
    body: (
      <div style={bodyStyle}>
        <p>• 通过 <strong>javac</strong> 将 .java 编译成 .class <strong>字节码</strong>（与平台无关的中间代码）。</p>
        <p>• .class 包含<strong>指令、常量池、元数据</strong>，供 JVM 加载和执行。</p>
        <pre style={{ background: 'rgba(0,0,0,0.3)', padding: 10, borderRadius: 8, fontSize: 16, marginTop: 10 }}>
          Main.java  --[javac]--&gt;  Main.class
        </pre>
        <p style={{ marginTop: 10, fontSize: 20 }}>“一次编写，到处运行”：同一份字节码由各平台 JVM 执行。</p>
      </div>
    ),
  },
  {
    title: '2.1 类加载器',
    body: (
      <div style={bodyStyleSmall}>
        <p>• <strong>启动类加载器</strong>：JDK 核心类（如 rt.jar）</p>
        <p>• <strong>扩展类加载器</strong>：扩展目录</p>
        <p>• <strong>应用程序类加载器</strong>：classpath 下应用类</p>
        <p>• <strong>自定义类加载器</strong>：继承 ClassLoader，热部署、隔离等</p>
      </div>
    ),
  },
  {
    title: '2.2 类加载过程',
    body: (
      <div style={bodyStyleSmall}>
        <p><strong>加载</strong>：查找字节流，创建 <code>java.lang.Class</code> 对象</p>
        <p><strong>链接</strong>：验证 → 准备（静态变量默认值）→ 解析（符号引用→直接引用）</p>
        <p><strong>初始化</strong>：执行 &lt;clinit&gt;，为静态变量赋初值</p>
      </div>
    ),
  },
  {
    title: '2.3 内存分配：运行时数据区',
    body: (
      <div style={bodyStyleSmall}>
        <p>• <strong>堆</strong>：对象实例、数组，线程共享，GC 管理</p>
        <p>• <strong>方法区</strong>：类信息、常量、静态变量、JIT 代码</p>
        <p>• <strong>虚拟机栈</strong>：每线程私有，存栈帧（局部变量、操作数栈、返回地址）</p>
        <p>• <strong>程序计数器</strong>：当前线程字节码指令地址</p>
        <p style={{ marginTop: 8 }}>另有<strong>本地方法栈</strong>供 Native 方法使用。</p>
      </div>
    ),
  },
  {
    title: '3. 执行',
    body: (
      <div style={bodyStyleSmall}>
        <p><strong>解释执行</strong>：逐条字节码→机器码，启动快，适合冷代码</p>
        <p><strong>JIT 编译</strong>：热点代码（频繁方法/循环）编译成机器码，效率高</p>
        <p style={{ marginTop: 12 }}>二者配合：先解释执行，JIT 后台编译热点，后续直接用机器码。</p>
      </div>
    ),
  },
  {
    title: '3.1 方法调用与栈帧',
    body: (
      <div style={bodyStyleSmall}>
        <p>方法被调用 → 创建<strong>栈帧</strong>压入虚拟机栈。栈帧含：</p>
        <p>• <strong>局部变量表</strong>：基本类型、引用（指向堆地址）</p>
        <p>• <strong>操作数栈</strong>、返回地址、动态链接</p>
        <p style={{ marginTop: 10 }}><strong>对象在堆</strong>，<strong>栈里存引用</strong>；局部变量表存“引用”，真正对象在堆上。</p>
      </div>
    ),
  },
  {
    title: '4. 小结',
    body: (
      <div style={bodyStyleSmall}>
        <pre style={{ background: 'rgba(0,0,0,0.3)', padding: 12, borderRadius: 8, fontSize: 16, whiteSpace: 'pre-wrap' }}>
{`.java → javac → .class
  → 类加载（加载→链接→初始化）
  → 解释 / JIT → 机器码
  → 栈帧入栈、对象在堆，CPU 执行`}
        </pre>
        <p style={{ marginTop: 12, fontSize: 20 }}>
          <strong>JVM 把与平台无关的字节码加载进内存、完成类加载与内存布局，再通过解释和 JIT 在具体平台上生成并执行机器码。</strong>
        </p>
      </div>
    ),
  },
];

export const ArticleSlides: React.FC = () => {
  const frame = useCurrentFrame();
  const slideIndex = Math.min(
    Math.floor(frame / FRAMES_PER_SLIDE),
    SLIDES.length - 1
  );
  const slide = SLIDES[slideIndex];

  return (
    <div
      style={{
        position: 'relative',
        width: '100%',
        height: '100%',
        background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)',
        color: '#e8e8e8',
        fontFamily: '"Noto Sans SC", "PingFang SC", sans-serif',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 48,
        boxSizing: 'border-box',
      }}
    >
      <h1 style={{ fontSize: 38, fontWeight: 700, margin: 0, textAlign: 'center' }}>
        {slide.title}
      </h1>
      <div style={{ textAlign: 'center' }}>
        {slide.body}
      </div>
      <div
        style={{
          position: 'absolute',
          bottom: 24,
          right: 32,
          fontSize: 16,
          opacity: 0.5,
        }}
      >
        {slideIndex + 1} / {SLIDES.length}
      </div>
    </div>
  );
};
