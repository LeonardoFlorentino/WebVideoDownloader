import { Panel, PlaylistCard, Info, Title, Status, Button, Empty } from './PlaylistPanel.styles';

export default function PlaylistPanel({ playlists, onDownload }: { playlists: Array<{ title: string, downloaded: boolean }>, onDownload: (title: string) => void }) {
  return (
    <Panel>
      {playlists.length === 0 ? (
        <Empty>Nenhuma playlist cadastrada.</Empty>
      ) : (
        playlists.map((pl, idx) => (
          <PlaylistCard key={idx}>
            <Info>
              <Title>{pl.title}</Title>
              <Status downloaded={pl.downloaded}>{pl.downloaded ? 'Baixada' : 'Pendente'}</Status>
            </Info>
            <Button disabled={pl.downloaded} onClick={() => onDownload(pl.title)}>
              Baixar
            </Button>
          </PlaylistCard>
        ))
      )}
    </Panel>
  );
}
