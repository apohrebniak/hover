package com.github.apohrebniak.hoverui;

import com.github.apohrebniak.hoverui.domain.MemberEntity;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.core.ParameterizedTypeReference;
import org.springframework.http.HttpMethod;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestTemplate;

import java.net.URI;
import java.util.List;

@Component
public class KvNodeService {

  @Autowired
  private RestTemplate restTemplate;

  public List<MemberEntity> getMembers(URI uri) { //TODO: params
    try {
      return restTemplate.exchange(uri,
          HttpMethod.GET,
          null,
          new ParameterizedTypeReference<List<MemberEntity>>() {
          }).getBody();
    } catch (Exception e) {
      return null;
    }
  }
}
