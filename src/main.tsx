import {createRoot} from 'react-dom/client';
import App from './App.tsx';
import './index.css';

if (new URLSearchParams(window.location.search).get('window') === 'pie') {
  document.documentElement.classList.add('pie-window');
}

createRoot(document.getElementById('root')!).render(<App />);
