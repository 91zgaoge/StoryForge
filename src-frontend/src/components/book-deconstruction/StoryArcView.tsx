import { GitBranch, Mountain, Zap } from 'lucide-react';
import type { ReferenceBook } from '@/types/book-deconstruction';

interface StoryArcViewProps {
  book: ReferenceBook;
}

export function StoryArcView({ book }: StoryArcViewProps) {
  const parseStoryArc = () => {
    if (!book.story_arc) return null;
    try {
      return JSON.parse(book.story_arc) as {
        main_arc: string;
        sub_arcs: string[];
        climaxes: string[];
        turning_points: string[];
      };
    } catch {
      return null;
    }
  };

  const arc = parseStoryArc();

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2 mb-4">
        <GitBranch className="w-5 h-5 text-cinema-gold" />
        <h3 className="text-lg font-medium text-white">故事线</h3>
      </div>

      {/* 主线 */}
      {arc?.main_arc && (
        <div className="bg-cinema-900 border border-cinema-800 rounded-xl p-4">
          <h4 className="text-sm font-medium text-cinema-gold mb-2">主线故事</h4>
          <p className="text-sm text-gray-300 leading-relaxed">{arc.main_arc}</p>
        </div>
      )}

      {/* 剧情概要 */}
      {book.plot_summary && (
        <div className="bg-cinema-900 border border-cinema-800 rounded-xl p-4">
          <h4 className="text-sm font-medium text-cinema-gold mb-2">剧情概要</h4>
          <p className="text-sm text-gray-300 leading-relaxed">{book.plot_summary}</p>
        </div>
      )}

      {/* 高潮点 */}
      {arc?.climaxes && arc.climaxes.length > 0 && (
        <div className="bg-cinema-900 border border-cinema-800 rounded-xl p-4">
          <div className="flex items-center gap-2 mb-3">
            <Mountain className="w-4 h-4 text-red-400" />
            <h4 className="text-sm font-medium text-red-400">高潮点</h4>
          </div>
          <div className="space-y-2">
            {arc.climaxes.map((climax, i) => (
              <div key={i} className="flex items-start gap-2">
                <span className="text-xs text-red-400/60 mt-0.5">{i + 1}.</span>
                <p className="text-sm text-gray-300">{climax}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 转折点 */}
      {arc?.turning_points && arc.turning_points.length > 0 && (
        <div className="bg-cinema-900 border border-cinema-800 rounded-xl p-4">
          <div className="flex items-center gap-2 mb-3">
            <Zap className="w-4 h-4 text-yellow-400" />
            <h4 className="text-sm font-medium text-yellow-400">转折点</h4>
          </div>
          <div className="space-y-2">
            {arc.turning_points.map((point, i) => (
              <div key={i} className="flex items-start gap-2">
                <span className="text-xs text-yellow-400/60 mt-0.5">{i + 1}.</span>
                <p className="text-sm text-gray-300">{point}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 支线 */}
      {arc?.sub_arcs && arc.sub_arcs.length > 0 && (
        <div className="bg-cinema-900 border border-cinema-800 rounded-xl p-4">
          <h4 className="text-sm font-medium text-blue-400 mb-3">支线故事</h4>
          <div className="space-y-2">
            {arc.sub_arcs.map((sub, i) => (
              <div key={i} className="flex items-start gap-2">
                <span className="text-xs text-blue-400/60 mt-0.5">{i + 1}.</span>
                <p className="text-sm text-gray-300">{sub}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {!arc && !book.plot_summary && (
        <div className="text-center py-8 text-gray-500">暂无故事线数据</div>
      )}
    </div>
  );
}
