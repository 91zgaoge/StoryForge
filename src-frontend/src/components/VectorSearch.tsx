import { useState } from 'react';
import { Search, X, Sparkles } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useVectorSearch } from '@/hooks/useVectorSearch';

interface VectorSearchProps {
  storyId: string;
}

export function VectorSearch({ storyId }: VectorSearchProps) {
  const [query, setQuery] = useState('');
  const { results, isLoading, search, clearResults } = useVectorSearch();

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    search({ story_id: storyId, query, top_k: 5 });
  };

  return (
    <div className="space-y-4">
      <form onSubmit={handleSearch} className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索章节内容..."
            className="w-full pl-10 pr-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white placeholder-gray-500 focus:border-cinema-gold focus:outline-none"
          />
          {query && (
            <button
              type="button"
              onClick={() => {
                setQuery('');
                clearResults();
              }}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
        <Button
          type="submit"
          variant="primary"
          isLoading={isLoading}
          className="gap-2"
        >
          <Sparkles className="w-4 h-4" />
          搜索
        </Button>
      </form>

      {results.length > 0 && (
        <div className="space-y-3">
          <p className="text-sm text-gray-400">找到 {results.length} 个相关结果</p>
          {results.map((result) => (
            <Card key={result.id} className="hover:border-cinema-gold/30 transition-colors">
              <CardContent className="p-4">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1">
                    <p className="text-sm text-cinema-gold mb-1">
                      第 {result.chapter_number} 章
                    </p>
                    <p className="text-gray-300 text-sm line-clamp-3">{result.text}</p>
                  </div>
                  <span className="text-xs text-gray-500 shrink-0">
                    相关度: {(result.score * 100).toFixed(1)}%
                  </span>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
