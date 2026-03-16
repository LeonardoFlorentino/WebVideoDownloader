import React from 'react';
import type{ VideoInfo } from '../types/video';
import VideoListItem from './VideoListItem';

interface VideoListProps {
  videos: VideoInfo[];
  onOpenFolder: (video: VideoInfo) => void;
  onShowDetails: (video: VideoInfo) => void;
}

const VideoList: React.FC<VideoListProps> = ({ videos, onOpenFolder, onShowDetails }) => (
  <div>
    {videos.map(video => (
      <VideoListItem
        key={video.path}
        video={video}
        onOpenFolder={onOpenFolder}
        onShowDetails={onShowDetails}
      />
    ))}
  </div>
);

export default VideoList;
