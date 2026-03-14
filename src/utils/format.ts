export function formatAbsoluteTime(dateString: string): string {
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) {
    return 'Invalid date';
  }

  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');

  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
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
    return remainingMinutes > 0
      ? `${diffHours} hours ${remainingMinutes} minutes`
      : `${diffHours} hours`;
  }

  if (diffMinutes > 0) {
    const remainingSeconds = diffSeconds % 60;
    return remainingSeconds > 0
      ? `${diffMinutes} minutes ${remainingSeconds} seconds`
      : `${diffMinutes} minutes`;
  }

  return `${diffSeconds} seconds`;
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

export function formatOnceAt(onceAt: string | undefined): string {
  if (!onceAt) {
    return 'No schedule';
  }

  const date = new Date(onceAt);
  if (Number.isNaN(date.getTime())) {
    return 'One-time (invalid date)';
  }

  return `One-time at ${date.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })}`;
}
