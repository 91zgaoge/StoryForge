import { Reveal, SectionHeader } from "./Reveal";

const FREE_FEATURES = [
  "幕前沉浸写作 · 单章续写",
  "场景、角色与知识图谱管理",
  "智能排版与斜杠命令",
  "故事大纲与世界观设定",
];

const PRO_FEATURES = [
  "免费版全部功能",
  "自动续写 —— 长篇持续创作不断档",
  "智能修改 —— 基于故事设定的全文级润色",
  "拆书分析 —— 解构参考书，反哺你的故事",
  "指导书提炼 —— 上传创作指导书，自动提炼创作方法论",
  "创作 Pipeline —— Refine / Review / Finalize 全流程",
];

function Check() {
  return (
    <svg
      className="mt-0.5 h-4 w-4 shrink-0 text-moss"
      viewBox="0 0 16 16"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="M3 8.5l3.5 3.5L13 4.5"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function PricingSection() {
  return (
    <section id="pricing" className="relative px-6 py-24 md:py-36">
      <div className="mx-auto max-w-[1080px]">
        <SectionHeader
          kicker="价格"
          title="免费开始，需要时再升级"
          lead="免费版就能完整地写一部小说；Pro 解锁自动化与深度分析能力，把重复劳动交给 AI。"
        />

        <div className="grid gap-5 md:grid-cols-2">
          {/* 免费版 */}
          <Reveal>
            <div className="surface-1 flex h-full flex-col rounded-lg border border-subtle p-8">
              <p className="text-xs tracking-[0.2em] text-dim">FREE</p>
              <h3 className="mt-2 text-xl text-paper">免费版</h3>
              <div className="mt-4 flex items-baseline gap-1">
                <span className="text-3xl text-paper tabular-nums">¥0</span>
                <span className="text-sm text-dim">/ 永久</span>
              </div>
              <ul className="mt-6 space-y-3">
                {FREE_FEATURES.map((f) => (
                  <li
                    key={f}
                    className="flex items-start gap-2.5 text-sm text-mist"
                  >
                    <Check />
                    <span>{f}</span>
                  </li>
                ))}
              </ul>
              <a
                href="#download"
                className="mt-8 inline-flex items-center justify-center rounded-md border border-subtle px-4 py-2.5 text-sm text-mist transition-colors hover:border-moss hover:text-paper"
              >
                免费下载
              </a>
            </div>
          </Reveal>

          {/* Pro */}
          <Reveal>
            <div className="surface-1 relative flex h-full flex-col rounded-lg border border-moss/50 p-8">
              <span className="absolute right-6 top-6 rounded-full bg-moss/15 px-2.5 py-1 text-[11px] font-medium text-moss-soft">
                早鸟价
              </span>
              <p className="text-xs tracking-[0.2em] text-moss">PRO</p>
              <h3 className="mt-2 text-xl text-paper">专业版</h3>
              <div className="mt-4 flex items-baseline gap-1">
                <span className="text-3xl text-paper tabular-nums">¥19</span>
                <span className="text-sm text-dim">/ 月 · 随时可退订</span>
              </div>
              <ul className="mt-6 space-y-3">
                {PRO_FEATURES.map((f) => (
                  <li
                    key={f}
                    className="flex items-start gap-2.5 text-sm text-mist"
                  >
                    <Check />
                    <span>{f}</span>
                  </li>
                ))}
              </ul>
              <a
                href="#download"
                className="mt-8 inline-flex items-center justify-center rounded-md bg-moss px-4 py-2.5 text-sm font-medium text-white transition-opacity hover:opacity-90"
              >
                下载后升级 Pro
              </a>
            </div>
          </Reveal>
        </div>

        <Reveal className="mt-10">
          <p className="text-sm text-dim">
            订阅在应用内一键完成；升级立即生效，退订后已生成的内容与方法论资产全部保留。
          </p>
        </Reveal>
      </div>
    </section>
  );
}
