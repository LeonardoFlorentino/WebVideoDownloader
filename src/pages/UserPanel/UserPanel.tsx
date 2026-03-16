
import { Container, Card, Header, IconCircle, Username, Button, Section, Title } from './UserPanel.styles';
import { TopBar, NavButton } from '../Home/Home.styles';
import { useNavigate } from 'react-router-dom';

export default function UserPanel({ username }: { username: string }) {
  const navigate = useNavigate();

  const handleLogout = () => {
    navigate('/login');
  };
  const handleGoHome = () => {
    navigate('/');
  };

  return (
    <Container style={{ minHeight: '100vh', width: '100%', display: 'flex', flexDirection: 'column' }}>
      <TopBar style={{ justifyContent: 'space-between' }}>
        <div>
          <NavButton onClick={handleGoHome}>Home</NavButton>
        </div>
        <div>
          <NavButton onClick={handleLogout}>Logout</NavButton>
        </div>
      </TopBar>
      <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', width: '100%' }}>
        <Card>
          <Header>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <IconCircle>
                <svg xmlns='http://www.w3.org/2000/svg' className='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='white'><path strokeLinecap='round' strokeLinejoin='round' strokeWidth={2} d='M16 21v-2a4 4 0 00-8 0v2M12 11a4 4 0 100-8 4 4 0 000 8z' /></svg>
              </IconCircle>
              <Username>Bem-vindo, {username}</Username>
            </div>
          </Header>
          <Section>
            <Title>Suas Playlists</Title>
            {/* TODO: Listar playlists do usuário */}
          </Section>
        </Card>
      </div>
    </Container>
  );
}
