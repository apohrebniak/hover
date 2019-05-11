package com.github.apohrebniak.hoverui.domain;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Data;

@Data
public class MemberEntity {
  private String id;
  @JsonProperty("http_address")
  private Address httpAddress;
  @JsonProperty("hover_node")
  private Node hoverNode;

  @Data
  public static class Address {
    private String ip;
    private Integer port;
  }

  @Data
  public static class Node {
    private String id;
    private Address addr;
  }
}