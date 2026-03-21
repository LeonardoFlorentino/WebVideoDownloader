export interface Download {
  id: number;
  url: string;
  ext: string;
  filename: string;
  progress: number;
  status: string;
  canceled: boolean;
  playlist?: string; // nome da playlist ou "" se não houver
  total?: number; // total em bytes, opcional
}
