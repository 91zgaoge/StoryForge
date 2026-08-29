export type StoryFormat = 'novel' | 'short_drama';

export function StoryFormatFields({
  format,
  onFormatChange,
}: {
  format: StoryFormat;
  onFormatChange: (format: StoryFormat) => void;
}) {
  return (
    <>
      <div>
        <label className="block text-sm text-gray-400 mb-1">体裁</label>
        <select
          data-testid="story-format-select"
          value={format}
          onChange={e => onFormatChange(e.target.value as StoryFormat)}
          className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
        >
          <option value="novel">长篇</option>
          <option value="short_drama">短剧</option>
        </select>
      </div>
      {format === 'short_drama' && (
        <div data-testid="production-constraints" className="space-y-2">
          <label className="block text-sm text-gray-400">制作限制（可选）</label>
          <input
            name="episodes"
            type="number"
            min={1}
            placeholder="集数"
            className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
          />
          <input
            name="seconds_per_episode"
            type="number"
            min={1}
            placeholder="单集秒数"
            className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
          />
          <input
            name="scene_cap"
            type="number"
            min={1}
            placeholder="场景上限"
            className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
          />
        </div>
      )}
    </>
  );
}

export function productionConstraintsJson(form: FormData): string | undefined {
  const episodes = String(form.get('episodes') ?? '').trim();
  const seconds = String(form.get('seconds_per_episode') ?? '').trim();
  const cap = String(form.get('scene_cap') ?? '').trim();
  if (!episodes && !seconds && !cap) {
    return undefined;
  }
  const num = (s: string) => {
    const n = Number(s);
    return Number.isFinite(n) && n > 0 ? n : undefined;
  };
  return JSON.stringify({
    episodes: num(episodes),
    seconds_per_episode: num(seconds),
    scene_cap: num(cap),
  });
}
