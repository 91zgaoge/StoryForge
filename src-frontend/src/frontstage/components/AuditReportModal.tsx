/**
 * AuditReportModal — 智能输入审计意图的报告弹窗
 *
 * v0.31.x: 智能输入框中的非散文审计意图（如"审计这一幕"）由后端自动路由到
 * 专用审计路径，报告以 result_kind='audit_report' 返回。前端以此弹窗展示
 * 报告全文，而不是把报告追加进手稿。
 */

import React from 'react';
import { ClipboardCheck, X } from 'lucide-react';

export interface AuditReportModalProps {
  isOpen: boolean;
  report: string;
  onClose: () => void;
}

export const AuditReportModal: React.FC<AuditReportModalProps> = ({ isOpen, report, onClose }) => {
  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-[10000] flex items-center justify-center"
      style={{ backgroundColor: 'rgba(0,0,0,0.55)' }}
      onClick={onClose}
    >
      <div
        className="rounded-2xl p-6 max-w-2xl w-full mx-4 shadow-2xl relative flex flex-col"
        style={{
          background: 'var(--parchment, #f5f0e1)',
          border: '1px solid var(--warm-sand, #d4c5a9)',
          color: 'var(--charcoal, #2c2c2c)',
          maxHeight: '80vh',
        }}
        onClick={e => e.stopPropagation()}
      >
        <button
          className="absolute top-4 right-4 p-1 rounded-full opacity-70 hover:opacity-100 transition-opacity"
          onClick={onClose}
          aria-label="关闭"
        >
          <X size={18} />
        </button>

        <div className="flex items-center gap-3 mb-4">
          <div
            className="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center"
            style={{ background: 'rgba(91,140,90,0.12)' }}
          >
            <ClipboardCheck size={20} color="var(--accent, #5b8c5a)" />
          </div>
          <h3 className="text-lg font-bold" style={{ color: 'var(--charcoal, #2c2c2c)' }}>
            审计报告
          </h3>
        </div>

        <div
          className="rounded-lg p-4 mb-5 text-sm leading-relaxed overflow-y-auto whitespace-pre-wrap break-words flex-1"
          style={{
            background: 'rgba(255,255,255,0.55)',
            border: '1px solid var(--warm-sand, #d4c5a9)',
          }}
        >
          {report}
        </div>

        <div className="flex justify-end">
          <button
            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors text-white"
            style={{ background: 'var(--accent, #5b8c5a)' }}
            onClick={onClose}
          >
            关闭
          </button>
        </div>
      </div>
    </div>
  );
};

export default AuditReportModal;
