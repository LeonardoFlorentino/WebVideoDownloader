import React from 'react';
import type { Download } from '../types/download';

interface DownloadListProps {
  downloads: Download[];
  onCancel: (id: number) => void;
}

const DownloadList: React.FC<DownloadListProps> = ({ downloads, onCancel }) => (
  <div className="download-list">
    {downloads.map(d => (
      <div key={d.id} className="download-list-item">
        <div className="download-info">
          <div className="download-title">{d.filename}</div>
          <div className="download-status">{d.status}</div>
          <div className="download-progress-bar">
            <div
              className="download-progress"
              style={{ width: `${d.progress}%`, background: d.status === 'Cancelado' ? '#b71c1c' : '#4caf50' }}
            />
            <span className="download-progress-text">{d.progress}%</span>
          </div>
        </div>
        {d.status === 'Baixando' && (
          <button className="download-cancel" onClick={() => onCancel(d.id)} title="Cancelar download">×</button>
        )}
      </div>
    ))}
  </div>
);

export default DownloadList;
