// Step 2: the video that was found: thumbnail, title, creator, date, length, views, description.
// (The thumbnail spot is where a preview player would go, if clips are added later.)

import { useState } from "react";
import { api, type VideoInfo } from "../api";
import { formatDuration, formatUploadDate, formatViews } from "../format";

export default function VideoCard({ video }: { video: VideoInfo }) {
  const [showDescription, setShowDescription] = useState(false);
  const facts = [formatUploadDate(video.uploadDate), formatDuration(video.durationSeconds), formatViews(video.viewCount)]
    .filter(Boolean)
    .join(" · ");
  const channelUrl = video.channelUrl;

  return (
    <section className="panel">
      <span className="label">Video</span>
      <div className="card">
        {video.thumbnail ? (
          <img className="thumbnail" src={video.thumbnail} alt="" referrerPolicy="no-referrer" />
        ) : (
          <div className="thumbnail" />
        )}
        <div className="details">
          <button type="button" className="title-link" onClick={() => api.openLink(video.url)} title="Open on YouTube">
            {video.title}
          </button>
          {channelUrl ? (
            <button type="button" className="subtle-link" onClick={() => api.openLink(channelUrl)} title="Open channel">
              {video.channel}
            </button>
          ) : (
            <span className="muted">{video.channel}</span>
          )}
          {facts && <span className="muted">{facts}</span>}
        </div>
      </div>
      {video.description && (
        <div className="description">
          <button
            type="button"
            className="secondary disclosure"
            onClick={() => setShowDescription(!showDescription)}
            aria-expanded={showDescription}
            aria-controls="video-description"
          >
            <span className="chevron" aria-hidden="true" />
            {showDescription ? "Hide description" : "Show description"}
          </button>
          {showDescription && (
            <p id="video-description" className="description-text">
              {video.description}
            </p>
          )}
        </div>
      )}
    </section>
  );
}
