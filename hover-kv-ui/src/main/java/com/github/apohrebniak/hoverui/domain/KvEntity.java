package com.github.apohrebniak.hoverui.domain;

import lombok.Builder;
import lombok.Data;

@Data
@Builder
public class KvEntity {
  private String key;
  private String value;
}
