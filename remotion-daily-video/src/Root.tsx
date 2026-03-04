import { Composition } from 'remotion';
import { ArticleSlides } from './ArticleSlides';

const FPS = 30;
const DURATION_IN_FRAMES = FPS * 38;
// 抖音竖版 9:16
const WIDTH = 1080;
const HEIGHT = 1920;

export const RemotionRoot = () => {
  return (
    <Composition
      id="ArticleSlides"
      component={ArticleSlides}
      durationInFrames={DURATION_IN_FRAMES}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
      defaultProps={{}}
    />
  );
};
