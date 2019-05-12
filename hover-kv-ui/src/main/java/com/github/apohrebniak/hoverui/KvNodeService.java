package com.github.apohrebniak.hoverui;

import com.github.apohrebniak.hoverui.domain.KvEntity;
import com.github.apohrebniak.hoverui.domain.MemberEntity;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.core.ParameterizedTypeReference;
import org.springframework.http.HttpEntity;
import org.springframework.http.HttpMethod;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClientException;
import org.springframework.web.client.RestTemplate;

import java.net.URI;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

@Component
public class KvNodeService {

  @Autowired
  private RestTemplate restTemplate;

  public List<MemberEntity> getMembers(String host, Integer port) { //TODO: params
    try {
      URI uri = URI.create("http://" + host + ":" + port + "/members");
      return restTemplate.exchange(uri,
          HttpMethod.GET,
          null,
          new ParameterizedTypeReference<List<MemberEntity>>() {
          }).getBody();
    } catch (Exception e) {
      return null;
    }
  }

  public Optional<List<KvEntity>> getKvAll(String host, Integer port) {
    try {
      URI uri = URI.create("http://" + host + ":" + port + "/kv");
      return Optional.ofNullable(restTemplate.exchange(uri,
          HttpMethod.GET,
          null,
          new ParameterizedTypeReference<Map<String, String>>() {
          }).getBody())
          .map(map -> map.entrySet()
              .stream()
              .map(e -> KvEntity.builder()
                  .key(e.getKey())
                  .value(e.getValue())
                  .build())
              .collect(Collectors.toList()));
    } catch (Exception e) {
      return Optional.empty();
    }
  }

  public boolean deleteKv(String host, Integer port, KvEntity item) {
    try {
      restTemplate.delete(URI.create("http://" + host + ":" + port.toString() + "/kv/" + item.getKey()));
    } catch (RestClientException ex) {
      return false;
    }
    return true;
  }

  public boolean addKv(String host, Integer port, KvEntity item) {
    try {
      URI uri = URI.create("http://" + host + ":" + port + "/kv/" + item.getKey());
      HttpEntity<KvEntity> request = new HttpEntity<>(item);
      restTemplate.exchange(uri, HttpMethod.POST, request, Object.class);
    } catch (RestClientException ex) {
      return false;
    }
    return true;
  }
}
