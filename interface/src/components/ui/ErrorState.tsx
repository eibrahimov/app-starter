export function ErrorState({ message }: { message: string }) {
  // role="alert" gives the message an assertive live region so screen readers
  // announce it when an error replaces the list.
  return (
    <p role="alert" className="text-sm text-destructive">
      {message}
    </p>
  );
}
