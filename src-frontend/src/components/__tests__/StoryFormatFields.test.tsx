import { it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { StoryFormatFields, type StoryFormat } from '../StoryFormatFields';

function Harness({ initial = 'novel' as StoryFormat }) {
  const [format, setFormat] = useState<StoryFormat>(initial);
  return <StoryFormatFields format={format} onFormatChange={setFormat} />;
}

it('制作限制只在短剧时显示', async () => {
  const user = userEvent.setup();
  render(<Harness />);
  expect(screen.queryByTestId('production-constraints')).not.toBeInTheDocument();
  await user.selectOptions(screen.getByTestId('story-format-select'), 'short_drama');
  expect(screen.getByTestId('production-constraints')).toBeInTheDocument();
  await user.selectOptions(screen.getByTestId('story-format-select'), 'novel');
  expect(screen.queryByTestId('production-constraints')).not.toBeInTheDocument();
});
