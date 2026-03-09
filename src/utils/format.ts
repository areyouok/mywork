export function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffHours = diffMs / (1000 * 60 * 60);
  const diffDays = diffMs / (1000 * 60 * 60 * 24);

  if (diffHours < 1) {
    return 'less than 1 minute ago';
  }
  if (diffHours < 2) {
    return '1 hour ago';
  }
  if (diffHours < 24) {
    return `${Math.floor(diffHours)} hours ago`;
  }
  if (diffDays < 7) {
    return date.toLocaleString('en-US', {
      weekday: 'short',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export function formatDuration(startedAt: string, finishedAt: string): string {
  const start = new Date(startedAt);
  const finish = new Date(finishedAt);
  const diffMs = finish.getTime() - start.getTime();

  if (diffMs < 1000) {
    return '<1s';
  }

  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);

  if (diffHours > 0) {
    const remainingMinutes = diffMinutes % 60;
    return remainingMinutes > 0 ? `${diffHours}h ${remainingMinutes}m` : `${diffHours}h`;
  }

  if (diffMinutes > 0) {
    const remainingSeconds = diffSeconds % 60;
    return remainingSeconds > 0 ? `${diffMinutes}m ${remainingSeconds}s` : `${diffMinutes}m`;
  }

  return `${diffSeconds}s`;
}

export function formatSimpleSchedule(simpleSchedule: string | undefined): string {
  if (!simpleSchedule) {
    return 'No schedule';
  }

  try {
    const schedule = JSON.parse(simpleSchedule);

    const dayMap: Record<string, string> = {
      monday: 'Mon',
      tuesday: 'Tue',
      wednesday: 'Wed',
      thursday: 'Thu',
      friday: 'Fri',
      saturday: 'Sat',
      sunday: 'Sun',
    };

    if (schedule.type === 'interval') {
      return `Every ${schedule.value} ${schedule.unit}`;
    }
    if (schedule.type === 'daily') {
      return `Daily at ${schedule.time}`;
    }
    if (schedule.type === 'weekly') {
      const day = dayMap[schedule.day.toLowerCase()] || schedule.day;
      return `${day} at ${schedule.time}`;
    }

    return JSON.stringify(schedule);
  } catch {
    return 'Custom schedule';
  }
}
