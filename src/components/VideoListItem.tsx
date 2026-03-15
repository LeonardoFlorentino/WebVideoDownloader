import React from 'react';
import type { VideoInfo } from '../types/video';

interface VideoListItemProps {
  video: VideoInfo;
  onOpenFolder: (video: VideoInfo) => void;
  onShowDetails: (video: VideoInfo) => void;
}

const VideoListItem: React.FC<VideoListItemProps> = ({ video, onOpenFolder, onShowDetails }) => (
  <div className="video-list-item">
    <img
      src={video.thumbnail || '/default-thumb.png'}
      alt={video.name}
      className="video-thumb"
      width={96}
      height={54}
      style={{ borderRadius: 8, objectFit: 'cover', marginRight: 16 }}
    />
    <div className="video-info">
      <div className="video-title">{video.name}</div>
      <div className="video-status">{video.status}</div>
      <div className="video-actions">
        <button onClick={() => onOpenFolder(video)}>Open Folder</button>
        <button onClick={() => onShowDetails(video)}>Details</button>
      </div>
    </div>
  </div>
);

export default VideoListItem;
