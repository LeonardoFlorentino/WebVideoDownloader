export interface User {
  username: string;
  playlists?: import('./playlist').Playlist[];
}
