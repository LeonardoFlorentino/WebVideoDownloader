export type Download = {
  id: number;
  url: string;
  ext: string;
  filename: string;
  progress: number;
  status: string;
  canceled: boolean;
};
