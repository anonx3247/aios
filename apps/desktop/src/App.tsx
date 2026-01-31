import { useEffect, useState } from 'react';
import { backend } from './lib/api';

function App() {
  const [status, setStatus] = useState<'connecting' | 'connected' | 'error'>(
    'connecting'
  );
  const [error, setError] = useState<string>('');

  useEffect(() => {
    let mounted = true;
    let retryCount = 0;
    const maxRetries = 10;

    async function connect() {
      while (mounted && retryCount < maxRetries) {
        try {
          await backend.healthCheck();
          if (mounted) {
            setStatus('connected');
            setError('');
          }
          return;
        } catch (err) {
          retryCount++;
          if (retryCount < maxRetries) {
            await new Promise((resolve) =>
              setTimeout(resolve, 1000 * retryCount)
            );
          } else {
            if (mounted) {
              setStatus('error');
              setError(`Failed after ${maxRetries} attempts`);
            }
          }
        }
      }
    }

    connect();
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <div className="min-h-screen bg-zinc-950 text-white flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-6xl font-bold mb-4">AIOS</h1>
        <p className="text-zinc-400 text-lg">
          Agent launcher and task manager
        </p>
        <div className="mt-4 text-sm">
          {status === 'connecting' && (
            <p className="text-yellow-500">Connecting to backend...</p>
          )}
          {status === 'connected' && (
            <p className="text-green-500">Backend connected</p>
          )}
          {status === 'error' && (
            <p className="text-red-500">Backend error: {error}</p>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
